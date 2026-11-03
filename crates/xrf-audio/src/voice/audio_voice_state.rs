use serde::{Deserialize, Serialize};

/// Where a voice is in its life, as the audio thread last reported it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AudioVoiceState {
  /// Asked for, not yet taken up by the audio thread: it starts with the next buffer the device asks for.
  Starting,
  Playing,
  Paused,
  /// Played to its end or stopped.
  Ended,
}

impl AudioVoiceState {
  pub(crate) fn to_code(self) -> u8 {
    match self {
      Self::Starting => 0,
      Self::Playing => 1,
      Self::Paused => 2,
      Self::Ended => 3,
    }
  }

  pub(crate) fn from_code(code: u8) -> Self {
    match code {
      1 => Self::Playing,
      2 => Self::Paused,
      3 => Self::Ended,
      _ => Self::Starting,
    }
  }
}
