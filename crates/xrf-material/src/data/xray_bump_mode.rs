use serde::Serialize;
use xrf_db::ThmBumpChunk;

/// Which bump shader family a declaration selects, `STextureParams::ETBumpMode` without the two values that mean no
/// bump at all (`ETextureParams.h`).
///
/// Parallax changes the pixel shader only: `uber_deffer.cpp` compiles `_steep` for it in HQ mode and the same
/// `_bump` variant as [`Self::Use`] otherwise. The inputs bound are the same pair either way.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum XrayBumpMode {
  Use,
  Parallax,
}

impl XrayBumpMode {
  /// The mode a chunk value selects, or `None` for the two that declare no bump.
  pub fn of(mode: u32) -> Option<Self> {
    match mode {
      ThmBumpChunk::MODE_USE => Some(Self::Use),
      ThmBumpChunk::MODE_USE_PARALLAX => Some(Self::Parallax),
      _ => None,
    }
  }
}
