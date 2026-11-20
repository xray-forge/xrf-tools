//! The configuration fields an inventory icon is declared with.
//!
//! Read from a section of `system.ltx` and its includes, where every item that shows an icon says which sheet it sits
//! on and which grid slot it occupies.

/// Marks a section as owning an inventory icon, gating both packing and unpacking.
pub(crate) const LTX_FIELD_INVENTORY_ICON: &str = "$inventory_icon";

/// Overrides where the icon of a section is read from, instead of `<section>.dds` in the source dir.
pub(crate) const LTX_FIELD_INVENTORY_ICON_PATH: &str = "$inventory_icon_path";

pub(crate) const LTX_FIELD_INV_GRID_X: &str = "inv_grid_x";

pub(crate) const LTX_FIELD_INV_GRID_Y: &str = "inv_grid_y";

pub(crate) const LTX_FIELD_INV_GRID_WIDTH: &str = "inv_grid_width";

pub(crate) const LTX_FIELD_INV_GRID_HEIGHT: &str = "inv_grid_height";

/// Inventory icon coordinates are expressed in grid cells rather than pixels, so every `inv_grid_*` value is
/// multiplied by this to reach the sprite sheet.
///
/// Note it is not a multiple of four, so icon boundaries do not align with the 4x4 blocks of a BC compressed sheet.
pub(crate) const INVENTORY_ICON_GRID_SQUARE_BASE: u32 = 50;
