use std::sync::Arc;

use crate::decode::decoded_sound::DecodedSound;
use crate::decode::stream::stream_reader::StreamReader;

/// Where a voice's frames come from.
#[derive(Debug)]
pub enum VoiceSource {
  /// A sound decoded whole, shared by every voice playing it.
  Decoded {
    sound: Arc<DecodedSound>,
    /// The frame to start at, clamped to the sound.
    start_frame: u64,
    /// Whether the voice starts over at the sound's end until stopped.
    is_looped: bool,
  },
  /// A sound decoded as it plays, read by this voice alone; it starts where its stream was opened and loops as the
  /// stream's control says.
  Streamed(StreamReader),
}

impl VoiceSource {
  /// The source frame the voice starts at.
  pub(crate) fn get_start_frame(&self) -> u64 {
    match self {
      Self::Decoded { sound, start_frame, .. } => (*start_frame).min(sound.get_frames_count() as u64),
      Self::Streamed(reader) => reader.get_start_frame(),
    }
  }
}
