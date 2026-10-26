use serde::{Deserialize, Serialize};

/// The layout of the X-Ray parameter block a sound's first Vorbis comment holds, by the `u32` it starts with.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum XRaySoundCommentVersion {
  /// Minimum and maximum distance and game type.
  V1,
  /// Adds the base volume.
  V2,
  /// Adds the maximum AI distance.
  V3,
}

impl XRaySoundCommentVersion {
  /// The version `CSoundManager::MakeGameSound` writes, `OGG_COMMENT_VERSION`.
  pub const CURRENT: Self = Self::V3;

  /// The version a comment starting with `code` is, or none for one `CSoundRender_Source::LoadWave` does not read.
  pub const fn from_code(code: u32) -> Option<Self> {
    match code {
      0x0001 => Some(Self::V1),
      0x0002 => Some(Self::V2),
      0x0003 => Some(Self::V3),
      _ => None,
    }
  }

  /// The version of the parameter block a comment holds, or none for a comment that is not one the engine reads.
  pub(crate) fn read(comment: &[u8]) -> Option<Self> {
    Self::from_code(u32::from_le_bytes(comment.get(..4)?.try_into().ok()?))
  }

  /// The `u32` a comment of this version starts with.
  pub const fn get_code(self) -> u32 {
    match self {
      Self::V1 => 0x0001,
      Self::V2 => 0x0002,
      Self::V3 => 0x0003,
    }
  }
}
