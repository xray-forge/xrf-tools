use serde::{Deserialize, Serialize};

/// Whether and how a descriptor asks the renderer to bind its bump pair, `STextureParams::ETBumpMode`
/// (`ETextureParams.h`).
///
/// The SDK trunk names three values and clamps anything below [`Self::None`] to it on load
/// (`ETextureParams.cpp`). [`Self::UseParallax`] is a fourth the trunk does not know and 483 descriptors of the
/// workspace corpus carry; it selects the same pair and differs only in the pixel shader the renderer compiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "u32", into = "u32")]
pub enum ThmBumpMode {
  /// `tbmResereved`, the pre-`Autogen` value the SDK reads as [`Self::None`].
  Reserved,
  None,
  Use,
  UseParallax,
  /// A value the SDK has no name for, kept as it was read so a descriptor written back says what it said.
  Unknown(u32),
}

impl ThmBumpMode {
  /// The modes an editor offers, which are the two that mean something plus the one that means off.
  pub const NAMED: [Self; 3] = [Self::None, Self::Use, Self::UseParallax];

  /// Whether the engine will try to resolve the declared name as a bump texture.
  pub const fn is_used(self) -> bool {
    matches!(self, Self::Use | Self::UseParallax)
  }

  /// Editor token for the mode, `tbmode_token` (`ETextureParams.cpp`), or the number for one the SDK has no name
  /// for.
  pub fn label(self) -> String {
    match self {
      Self::Reserved => String::from("Reserved"),
      Self::None => String::from("None"),
      Self::Use => String::from("Use"),
      Self::UseParallax => String::from("Use parallax"),
      Self::Unknown(raw) => raw.to_string(),
    }
  }
}

impl Default for ThmBumpMode {
  /// `STextureParams::STextureParams` sets `tbmNone` (`ETextureParams.h:119`).
  fn default() -> Self {
    Self::None
  }
}

impl From<u32> for ThmBumpMode {
  fn from(raw: u32) -> Self {
    match raw {
      0 => Self::Reserved,
      1 => Self::None,
      2 => Self::Use,
      3 => Self::UseParallax,
      raw => Self::Unknown(raw),
    }
  }
}

impl From<ThmBumpMode> for u32 {
  fn from(mode: ThmBumpMode) -> Self {
    match mode {
      ThmBumpMode::Reserved => 0,
      ThmBumpMode::None => 1,
      ThmBumpMode::Use => 2,
      ThmBumpMode::UseParallax => 3,
      ThmBumpMode::Unknown(raw) => raw,
    }
  }
}
