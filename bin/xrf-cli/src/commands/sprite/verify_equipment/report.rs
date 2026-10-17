use std::path::Path;

use serde::Serialize;
use xrf_report::Status;
use xrf_texture::EquipmentGridOverlap;

use crate::commands::sprite::verify_equipment::sprite_equipment_overlap_report::SpriteEquipmentOverlapReport;

/// The verdict `sprite verify-equipment` reached over an inventory icon grid.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteEquipmentVerifyReport {
  findings: Vec<SpriteEquipmentOverlapReport>,
  status: Status,
  subject: String,
}

impl SpriteEquipmentVerifyReport {
  pub fn new(subject: &Path, overlaps: &[EquipmentGridOverlap]) -> Self {
    Self {
      status: Status::from_is_valid(overlaps.is_empty()),
      findings: overlaps.iter().map(SpriteEquipmentOverlapReport::new).collect(),
      subject: xrf_utils::to_portable_path_string(subject),
    }
  }
}
