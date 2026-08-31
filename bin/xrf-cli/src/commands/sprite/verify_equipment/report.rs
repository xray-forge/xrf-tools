use std::path::Path;

use serde::Serialize;
use xrf_report::Status;
use xrf_texture::EquipmentGridOverlap;

/// One square of the inventory icon grid.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteGridCellReport {
  x: u32,
  y: u32,
}

/// Two sections whose inventory icon rectangles cover a common square.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteEquipmentOverlapReport {
  cell: SpriteGridCellReport,
  first: String,
  overlapping_cells: u32,
  second: String,
}

impl SpriteEquipmentOverlapReport {
  pub fn new(overlap: &EquipmentGridOverlap) -> Self {
    Self {
      cell: SpriteGridCellReport {
        x: overlap.cell.0,
        y: overlap.cell.1,
      },
      first: overlap.first.clone(),
      overlapping_cells: overlap.overlapping_cells,
      second: overlap.second.clone(),
    }
  }
}

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
