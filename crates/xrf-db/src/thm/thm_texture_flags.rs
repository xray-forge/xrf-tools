use serde::{Deserialize, Serialize};

use crate::thm::thm_texture_flag::ThmTextureFlag;

/// The texture param flag word, `Flags32 flags` of `STextureParams` (`ETextureParams.h`).
///
/// A word rather than a set of booleans: the SDK names twelve bits and a file may carry others, and a descriptor
/// written back has to say exactly what it said. [`Self::unnamed`] is what the word carries beyond the twelve, which
/// is a thing a surface can report rather than quietly drop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "u32", into = "u32")]
pub struct ThmTextureFlags(u32);

impl ThmTextureFlags {
  /// The word `STextureParams::STextureParams` starts a new descriptor with (`ETextureParams.h:115`).
  pub const DEFAULT: Self = Self::none()
    .with(ThmTextureFlag::GenerateMipMaps, true)
    .with(ThmTextureFlag::DitherColor, true);

  /// A word with no bit set.
  pub const fn none() -> Self {
    Self(0)
  }

  /// The word as the file stores it.
  pub const fn raw(self) -> u32 {
    self.0
  }

  /// Whether the flag is set.
  pub const fn has(self, flag: ThmTextureFlag) -> bool {
    self.0 & flag.bit() != 0
  }

  /// The same word with one flag set or cleared.
  pub const fn with(self, flag: ThmTextureFlag, is_set: bool) -> Self {
    if is_set {
      Self(self.0 | flag.bit())
    } else {
      Self(self.0 & !flag.bit())
    }
  }

  /// Sets or clears one flag in place.
  pub const fn set(&mut self, flag: ThmTextureFlag, is_set: bool) {
    *self = self.with(flag, is_set);
  }

  /// The named flags the word carries, in bit order.
  pub fn named(self) -> impl Iterator<Item = ThmTextureFlag> {
    ThmTextureFlag::NAMED.into_iter().filter(move |flag| self.has(*flag))
  }

  /// The bits the word carries that the SDK has no name for.
  pub fn unnamed(self) -> u32 {
    ThmTextureFlag::NAMED
      .into_iter()
      .fold(self.0, |remaining, flag| remaining & !flag.bit())
  }
}

impl From<u32> for ThmTextureFlags {
  fn from(raw: u32) -> Self {
    Self(raw)
  }
}

impl From<ThmTextureFlags> for u32 {
  fn from(flags: ThmTextureFlags) -> Self {
    flags.0
  }
}
