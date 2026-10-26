use serde::Serialize;
use xrf_thm::ThmBumpMode;

/// Which bump shader family a declaration selects, `STextureParams::ETBumpMode` without the two values that mean no
/// bump at all (`ETextureParams.h`).
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum XrayBumpMode {
  Use,
  Parallax,
}

impl XrayBumpMode {
  /// The mode a chunk value selects, or `None` for the values that declare no bump.
  pub fn of(mode: ThmBumpMode) -> Option<Self> {
    match mode {
      ThmBumpMode::Use => Some(Self::Use),
      ThmBumpMode::UseParallax => Some(Self::Parallax),
      _ => None,
    }
  }
}
