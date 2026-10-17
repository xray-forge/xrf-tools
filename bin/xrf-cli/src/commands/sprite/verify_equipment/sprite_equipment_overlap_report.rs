use serde::Serialize;
use xrf_texture::EquipmentGridOverlap;

use crate::commands::sprite::verify_equipment::sprite_grid_cell_report::SpriteGridCellReport;

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
      cell: SpriteGridCellReport::new(overlap.cell),
      first: overlap.first.clone(),
      overlapping_cells: overlap.overlapping_cells,
      second: overlap.second.clone(),
    }
  }
}
