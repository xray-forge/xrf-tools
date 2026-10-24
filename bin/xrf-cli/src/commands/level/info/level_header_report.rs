use serde::Serialize;
use xrf_level::LevelFile;

/// What the compiler stamped the level with.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelHeaderReport {
  xrlc_quality: u16,
  xrlc_version: u16,
}

impl LevelHeaderReport {
  pub fn of(level: &LevelFile) -> Self {
    Self {
      xrlc_quality: level.header.xrlc_quality,
      xrlc_version: level.header.xrlc_version,
    }
  }
}
