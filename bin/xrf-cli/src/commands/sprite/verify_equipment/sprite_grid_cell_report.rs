use serde::Serialize;

/// One square of the inventory icon grid.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteGridCellReport {
  x: u32,
  y: u32,
}

impl SpriteGridCellReport {
  pub fn new((x, y): (u32, u32)) -> Self {
    Self { x, y }
  }
}
