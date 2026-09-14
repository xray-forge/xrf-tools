use std::cmp::max;

use image::{ImageBuffer, Rgba, RgbaImage};
use serde::Serialize;
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};

use crate::equipment::{
  EquipmentGridRect, EquipmentSlotClaim, INVENTORY_ICON_GRID_SQUARE_BASE, LTX_FIELD_INVENTORY_ICON_PATH,
};
use xrf_dds::DDS_BLOCK_SIZE;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct InventorySpriteDescriptor {
  pub section: String,
  pub custom_icon: Option<String>,
  // X/Y/W/H are not absolute pixel units, just inventory boxes.
  pub x: u32,
  pub y: u32,
  pub w: u32,
  pub h: u32,
}

impl InventorySpriteDescriptor {
  pub fn new_list_from_ltx(ltx: &Ltx) -> Vec<Self> {
    let mut inventory_sections: Vec<Self> = Vec::new();

    for (section_name, section) in ltx.iter() {
      if let Some(inventory_section) = Self::new_optional_from_section(section_name, section) {
        inventory_sections.push(inventory_section);
      }
    }

    inventory_sections
  }

  /// Describe the inventory icon of a section, if it declares one.
  ///
  /// A section opts in with `$inventory_icon = true`. Placement alone is not enough here, because this decides what
  /// gets written: the grid fields are also inherited by variants that own no icon file of their own, and packing
  /// those would write art nobody authored. [`crate::EquipmentSlotOccupant`] asks the weaker question, for reading.
  pub fn new_optional_from_section<T>(section_name: T, section: &Section) -> Option<Self>
  where
    T: Into<String>,
  {
    if EquipmentSlotClaim::of_section(section) != Some(EquipmentSlotClaim::Declared) {
      return None;
    }

    let rect: EquipmentGridRect = EquipmentGridRect::new_optional_from_section(section)?;

    Some(Self {
      section: section_name.into(),
      custom_icon: section.get(LTX_FIELD_INVENTORY_ICON_PATH).map(Into::into),
      x: rect.x,
      y: rect.y,
      w: rect.w,
      h: rect.h,
    })
  }
}

impl InventorySpriteDescriptor {
  pub fn get_boundaries(&self) -> (u32, u32, u32, u32) {
    (
      self.x * INVENTORY_ICON_GRID_SQUARE_BASE,
      self.y * INVENTORY_ICON_GRID_SQUARE_BASE,
      self.w * INVENTORY_ICON_GRID_SQUARE_BASE,
      self.h * INVENTORY_ICON_GRID_SQUARE_BASE,
    )
  }
}

impl InventorySpriteDescriptor {
  /// Prepare combined equipment image base with suitable base size.
  pub fn create_equipment_sprite_base_for_ltx(ltx: &Ltx) -> XrfResult<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    let (max_width, max_height) = Self::get_equipment_sprite_boundaries_from_ltx(ltx);

    if max_width > 32 * 1024 || max_height > 32 * 1024 {
      Err(XrfError::new_texture_processing_error(format!(
        "Trying to create too large resulting dds file over 32k*32k ({max_width}x{max_height}), it is not supported",
      )))
    } else {
      Ok(RgbaImage::new(max_width, max_height))
    }
  }

  /// Smallest `DDS_BLOCK_SIZE`-aligned canvas that holds every icon claiming a grid slot.
  ///
  /// A grid square is 50 pixels, so a slot boundary lands on a whole block only every other column and
  /// row, and the rounding adds at most two pixels per axis. An already aligned canvas is returned
  /// untouched.
  pub fn get_equipment_sprite_boundaries_from_ltx(ltx: &Ltx) -> (u32, u32) {
    let mut max_width: u32 = 0;
    let mut max_height: u32 = 0;

    for (section_name, section) in ltx.iter() {
      if let Some(sprite) = Self::new_optional_from_section(section_name, section) {
        max_width = max((sprite.x + sprite.w) * INVENTORY_ICON_GRID_SQUARE_BASE, max_width);
        max_height = max((sprite.y + sprite.h) * INVENTORY_ICON_GRID_SQUARE_BASE, max_height);
      }
    }

    (
      max_width.next_multiple_of(DDS_BLOCK_SIZE),
      max_height.next_multiple_of(DDS_BLOCK_SIZE),
    )
  }
}
