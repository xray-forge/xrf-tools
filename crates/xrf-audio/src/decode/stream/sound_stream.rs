use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rtrb::{Producer, RingBuffer};
use xrf_error::{XrfError, XrfResult};

use crate::decode::sound_decoder::SoundDecoder;
use crate::decode::stream::sound_stream_start::SoundStreamStart;
use crate::decode::stream::stream_reader::StreamReader;
use crate::decode::stream::stream_shared::StreamShared;

/// Streams a long sound from its compressed bytes: a thread decodes ahead into a ring the voice reads, so music and
/// previews never hold their whole audio.
pub struct SoundStream;

impl SoundStream {
  /// Samples the ring holds: about 0.7 s of stereo at 48 kHz, enough to ride out a busy decoder thread.
  const RING_SAMPLES: usize = 1 << 16;

  /// How long the decoder thread waits once the ring is full.
  const FULL_WAIT: Duration = Duration::from_millis(2);

  /// Opens the sound in `bytes` at `start.frame` and starts decoding it; the reader goes to the voice that plays it.
  ///
  /// The sound is opened and sought before this returns, so a sound that cannot be read is refused here rather than
  /// heard as silence.
  ///
  /// # Errors
  ///
  /// Returns an error when [`SoundDecoder`] cannot open the sound or seek to the frame, or the thread cannot start.
  pub fn start(bytes: Arc<[u8]>, start: SoundStreamStart) -> XrfResult<StreamReader> {
    let mut decoder: SoundDecoder = SoundDecoder::open(bytes)?;
    let frame: u64 = start.frame.min(decoder.get_frames_count());

    decoder.seek(frame)?;

    let shared: Arc<StreamShared> = Arc::new(StreamShared::new(&decoder, start.is_looped));
    let (producer, consumer) = RingBuffer::new(Self::RING_SAMPLES);
    let thread_shared: Arc<StreamShared> = shared.clone();

    thread::Builder::new()
      .name(String::from("xrf-audio-stream"))
      .spawn(move || Self::run(decoder, producer, &thread_shared))
      .map_err(|error| XrfError::new_unexpected_error(format!("Could not start a sound stream: {error}")))?;

    Ok(StreamReader::new(consumer, shared, frame))
  }

  /// Decodes until the reader is dropped, the sound ends without looping, or the decoder fails.
  fn run(mut decoder: SoundDecoder, mut producer: Producer<f32>, shared: &StreamShared) {
    let mut pending: Vec<f32> = Vec::new();
    let mut offset: usize = 0;

    loop {
      if producer.is_abandoned() {
        return;
      }

      if offset == pending.len() {
        // The decoder replaces what `pending` held, emptying it at the end.
        offset = 0;

        match decoder.decode_next(&mut pending) {
          Ok(true) => {}
          Ok(false) if shared.is_looped() && decoder.get_frames_count() > 0 => {
            if let Err(error) = decoder.seek(0) {
              shared.fail(error.to_string());
              return;
            }
          }
          Ok(false) => {
            shared.end();
            return;
          }
          Err(error) => {
            shared.fail(error.to_string());
            return;
          }
        }

        continue;
      }

      let count: usize = producer.slots().min(pending.len() - offset);

      if count == 0 {
        thread::sleep(Self::FULL_WAIT);
        continue;
      }

      if let Ok(chunk) = producer.write_chunk_uninit(count) {
        offset += chunk.fill_from_iter(pending[offset..offset + count].iter().copied());
      }
    }
  }
}
