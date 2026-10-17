use serde::Serialize;

/// What the compiler stamped the level with.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelHeaderReport {
  pub xrlc_quality: u16,
  pub xrlc_version: u16,
}
