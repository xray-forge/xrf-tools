use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rtrb::{Producer, RingBuffer};
use xrf_error::{XrfError, XrfResult};

use crate::decode::sound_decoder::SoundDecoder;
use crate::decode::sound_format::SoundFormat;
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

  /// Starts decoding the sound in `bytes`, of `format`, from `start.frame` on a thread of its own; the reader goes to the
  /// voice that plays it.
  ///
  /// The thread opens the sound and seeks before it decodes, so no caller waits on a seek. A voice reading the stream
  /// is silent until the first frames come; a sound that cannot be opened, is not of `format`, or cannot be sought ends
  /// the stream, and [`crate::StreamControl::get_failure`] says why.
  ///
  /// # Errors
  ///
  /// Returns an error when the thread cannot start.
  pub fn start(bytes: Arc<[u8]>, format: SoundFormat, start: SoundStreamStart) -> XrfResult<StreamReader> {
    let frame: u64 = start.frame.min(format.frames_count);
    let shared: Arc<StreamShared> = Arc::new(StreamShared::new(format, start.is_looped));
    let (producer, consumer) = RingBuffer::new(Self::RING_SAMPLES);
    let thread_shared: Arc<StreamShared> = shared.clone();

    thread::Builder::new()
      .name(String::from("xrf-audio-stream"))
      .spawn(move || Self::open_and_run(bytes, frame, producer, &thread_shared))
      .map_err(|error| XrfError::new_unexpected_error(format!("Could not start a sound stream: {error}")))?;

    Ok(StreamReader::new(consumer, shared, frame))
  }

  fn open_and_run(bytes: Arc<[u8]>, frame: u64, producer: Producer<f32>, shared: &StreamShared) {
    match Self::open(bytes, frame, shared.format) {
      Ok(decoder) => Self::run(decoder, producer, shared),
      Err(error) => shared.fail(error.to_string()),
    }
  }

  fn open(bytes: Arc<[u8]>, frame: u64, format: SoundFormat) -> XrfResult<SoundDecoder> {
    let mut decoder: SoundDecoder = SoundDecoder::open(bytes)?;

    if decoder.get_format() != format {
      return Err(XrfError::new_read_error(format!(
        "The sound is {:?}, not the {format:?} it was started as",
        decoder.get_format()
      )));
    }

    decoder.seek(frame)?;

    Ok(decoder)
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
