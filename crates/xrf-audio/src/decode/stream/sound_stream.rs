use std::sync::Arc;
use std::thread;
use std::time::Duration;

use rtrb::RingBuffer;
use xrf_error::{XrfError, XrfResult};

use crate::decode::sound_format::SoundFormat;
use crate::decode::stream::sound_stream_start::SoundStreamStart;
use crate::decode::stream::stream_pump::StreamPump;
use crate::decode::stream::stream_pump_state::StreamPumpState;
use crate::decode::stream::stream_reader::StreamReader;
use crate::decode::stream::stream_shared::StreamShared;

/// Streams a long sound from its compressed bytes: a [`StreamPump`] decodes ahead into a ring the voice reads, so music
/// and previews never hold their whole audio.
pub struct SoundStream;

impl SoundStream {
  /// Samples the ring holds: about 0.7 s of stereo at 48 kHz, enough to ride out a busy decoder thread.
  const RING_SAMPLES: usize = 1 << 16;

  /// How long the decoder thread waits once the ring is full.
  const FULL_WAIT: Duration = Duration::from_millis(2);

  /// Starts decoding the sound in `bytes`, of `format`, from `start.frame` on a thread of its own; the reader goes to the
  /// voice that plays it.
  ///
  /// The thread opens and seeks, so no caller waits on a seek, and the voice is silent until the first frames come. A
  /// sound that cannot be opened or sought, or is not of `format`, ends the stream with
  /// [`crate::StreamControl::get_failure`] saying why.
  ///
  /// # Errors
  ///
  /// Returns an error when the thread cannot start.
  pub fn start(bytes: Arc<[u8]>, format: SoundFormat, start: SoundStreamStart) -> XrfResult<StreamReader> {
    let (reader, mut pump) = Self::start_pumped(bytes, format, start);

    thread::Builder::new()
      .name(String::from("xrf-audio-stream"))
      .spawn(move || {
        while pump.fill() == StreamPumpState::Full {
          thread::sleep(Self::FULL_WAIT);
        }
      })
      .map_err(|error| XrfError::new_unexpected_error(format!("Could not start a sound stream: {error}")))?;

    Ok(reader)
  }

  /// The same stream with its pump handed back rather than run on a thread: whoever holds it fills the ring.
  pub fn start_pumped(bytes: Arc<[u8]>, format: SoundFormat, start: SoundStreamStart) -> (StreamReader, StreamPump) {
    let frame: u64 = start.frame.min(format.frames_count);
    let shared: Arc<StreamShared> = Arc::new(StreamShared::new(format, start.is_looped));
    let (producer, consumer) = RingBuffer::new(Self::RING_SAMPLES);

    (
      StreamReader::new(consumer, Arc::clone(&shared), frame),
      StreamPump::new(bytes, frame, producer, shared),
    )
  }
}
