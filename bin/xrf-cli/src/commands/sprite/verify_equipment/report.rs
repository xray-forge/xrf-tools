use std::path::Path;

use serde::Serialize;
use xrf_error::XrfError;
use xrf_report::Status;
use xrf_texture::EquipmentGridOverlap;

use crate::commands::sprite::verify_equipment::sprite_equipment_overlap_report::SpriteEquipmentOverlapReport;

/// The verdict `sprite verify-equipment` reached over an inventory icon grid.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteEquipmentVerifyReport {
  findings: Vec<SpriteEquipmentOverlapReport>,
  /// Why the system ltx could not be read as one, where it could not: nothing in it was checked.
  #[serde(skip_serializing_if = "Option::is_none")]
  malformed: Option<String>,
  status: Status,
  subject: String,
}

impl SpriteEquipmentVerifyReport {
  pub fn new(subject: &Path, overlaps: &[EquipmentGridOverlap]) -> Self {
    Self {
      status: Status::from_is_valid(overlaps.is_empty()),
      findings: overlaps.iter().map(SpriteEquipmentOverlapReport::new).collect(),
      malformed: None,
      subject: xrf_utils::to_portable_path_string(subject),
    }
  }

  /// A system ltx that does not parse, judged failed.
  pub fn malformed(subject: &Path, error: &XrfError) -> Self {
    Self {
      status: Status::Failed,
      findings: Vec::new(),
      malformed: Some(error.to_string()),
      subject: xrf_utils::to_portable_path_string(subject),
    }
  }
}
