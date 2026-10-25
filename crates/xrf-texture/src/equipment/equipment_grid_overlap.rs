/// Two sections whose inventory icon rectangles cover a common cell.
#[derive(Clone, Debug, PartialEq)]
pub struct EquipmentGridOverlap {
  pub first: String,
  pub second: String,
  /// One cell both rectangles cover, enough to locate the clash on the sheet.
  pub cell: (u32, u32),
  pub overlapping_cells: u32,
}
