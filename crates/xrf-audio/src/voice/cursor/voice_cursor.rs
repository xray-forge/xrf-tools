use crate::decode::sound_channels::SoundChannels;
use crate::voice::cursor::decoded_cursor::DecodedCursor;
use crate::voice::cursor::stream_cursor::StreamCursor;
use crate::voice::voice_source::VoiceSource;

/// A voice's place in its source.
#[derive(Debug)]
pub(crate) enum VoiceCursor {
  Decoded(DecodedCursor),
  Streamed(StreamCursor),
}

impl VoiceCursor {
  pub(crate) fn get_sample_rate(&self) -> u32 {
    match self {
      Self::Decoded(cursor) => cursor.get_sample_rate(),
      Self::Streamed(cursor) => cursor.get_sample_rate(),
    }
  }

  /// The channels the source renders in: a mono source stays mono until the voice places it.
  pub(crate) fn get_channels(&self) -> SoundChannels {
    match self {
      Self::Decoded(cursor) => cursor.get_channels(),
      Self::Streamed(cursor) => cursor.get_channels(),
    }
  }

  pub(crate) fn get_frame(&self) -> u64 {
    match self {
      Self::Decoded(cursor) => cursor.get_frame(),
      Self::Streamed(cursor) => cursor.get_frame(),
    }
  }

  /// Writes the next frames, resampled by `step`, into `out` in the source's channels; `false` once it has played
  /// out.
  #[inline]
  pub(crate) fn render(&mut self, out: &mut [f32], step: f64) -> bool {
    match self {
      Self::Decoded(cursor) => cursor.render(out, step),
      Self::Streamed(cursor) => cursor.render(out, step),
    }
  }

  /// Moves on `frames` output frames resampled by `step`, rendering nothing; `false` once it has played out.
  pub(crate) fn skip(&mut self, frames: usize, step: f64) -> bool {
    match self {
      Self::Decoded(cursor) => cursor.skip(frames, step),
      Self::Streamed(cursor) => cursor.skip(frames, step),
    }
  }
}

impl From<VoiceSource> for VoiceCursor {
  fn from(source: VoiceSource) -> Self {
    match source {
      VoiceSource::Decoded {
        sound,
        start_frame,
        is_looped,
      } => Self::Decoded(DecodedCursor::new(sound, start_frame, is_looped)),
      VoiceSource::Streamed(reader) => Self::Streamed(StreamCursor::new(reader)),
    }
  }
}
