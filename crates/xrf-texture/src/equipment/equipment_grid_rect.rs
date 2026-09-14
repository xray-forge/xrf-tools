use xrf_ltx::Section;

use crate::equipment::{
  LTX_FIELD_INV_GRID_HEIGHT, LTX_FIELD_INV_GRID_WIDTH, LTX_FIELD_INV_GRID_X, LTX_FIELD_INV_GRID_Y,
};

/// Where a section sits on the inventory grid, in cells rather than pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EquipmentGridRect {
  pub(crate) x: u32,
  pub(crate) y: u32,
  pub(crate) w: u32,
  pub(crate) h: u32,
}

impl EquipmentGridRect {
  /// The slot a section places itself in, or `None` when it places itself nowhere usable.
  pub(crate) fn new_optional_from_section(section: &Section) -> Option<Self> {
    Some(Self {
      x: Self::read_cells(section, LTX_FIELD_INV_GRID_X)?,
      y: Self::read_cells(section, LTX_FIELD_INV_GRID_Y)?,
      w: Self::read_extent(section, LTX_FIELD_INV_GRID_WIDTH)?,
      h: Self::read_extent(section, LTX_FIELD_INV_GRID_HEIGHT)?,
    })
  }

  /// A coordinate in cells, or `None` when the field is absent or is not a whole number of them.
  fn read_cells(section: &Section, field: &str) -> Option<u32> {
    section.get(field)?.parse::<u32>().ok()
  }

  /// An extent in cells, which must cover at least one.
  fn read_extent(section: &Section, field: &str) -> Option<u32> {
    Self::read_cells(section, field).filter(|extent| *extent > 0)
  }
}
