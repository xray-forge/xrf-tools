use std::sync::Arc;

use crate::decode::decoded_sound::DecodedSound;
use crate::decode::sound_format::SoundFormat;

/// A sound as a scene's emitters play it: decoded whole, streamed from its bytes, still being read, or refused.
#[derive(Clone, Debug)]
pub(crate) enum BankedSound {
  Loading,
  Decoded(Arc<DecodedSound>),
  Streamed { bytes: Arc<[u8]>, format: SoundFormat },
  Refused(String),
}

impl BankedSound {
  /// How many channels it plays in, none until it is read or once it is refused.
  pub(crate) fn get_channel_count(&self) -> Option<usize> {
    match self {
      Self::Decoded(sound) => Some(sound.get_channels().get_count()),
      Self::Streamed { format, .. } => Some(format.channels.get_count()),
      Self::Loading | Self::Refused(_) => None,
    }
  }

  /// The bytes it holds in memory.
  pub(crate) fn get_size(&self) -> usize {
    match self {
      Self::Decoded(sound) => std::mem::size_of_val(sound.get_samples()),
      Self::Streamed { bytes, .. } => bytes.len(),
      Self::Loading | Self::Refused(_) => 0,
    }
  }

  /// Whether nothing but the bank holds what it read, so letting it go frees it.
  pub(crate) fn is_unheld(&self) -> bool {
    match self {
      Self::Decoded(sound) => Arc::strong_count(sound) == 1,
      Self::Streamed { bytes, .. } => Arc::strong_count(bytes) == 1,
      Self::Loading | Self::Refused(_) => false,
    }
  }
}
