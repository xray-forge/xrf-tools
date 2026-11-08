use std::path::Path;

use serde::Serialize;
use xrf_report::Report;

/// What `sprite verify-equipment` found in the icon grid, beside the system ltx it read the grid from.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SpriteVerifyEquipmentReport {
  system_ltx: String,
  #[serde(flatten)]
  report: Report,
}

impl SpriteVerifyEquipmentReport {
  pub(crate) fn new(system_ltx: &Path, report: Report) -> Self {
    Self {
      system_ltx: xrf_utils::to_portable_path_string(system_ltx),
      report,
    }
  }
}
