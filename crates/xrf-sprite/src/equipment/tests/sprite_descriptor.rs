use image::RgbaImage;
use xrf_error::XrfResult;
use xrf_ltx::Ltx;

use crate::equipment::InventorySpriteDescriptor;

fn descriptor_for(ltx: &str, section: &str) -> Option<InventorySpriteDescriptor> {
  let ltx: Ltx = Ltx::read_from_str(ltx).expect("test LTX is valid");

  InventorySpriteDescriptor::new_optional_from_section(section, &ltx[section])
}

#[test]
fn describes_sections_that_opt_in() {
  let descriptor: InventorySpriteDescriptor = descriptor_for(
    "[wpn_ak74]\n\
     $inventory_icon = true\n\
     inv_grid_x = 25\n\
     inv_grid_y = 4\n\
     inv_grid_width = 5\n\
     inv_grid_height = 2\n",
    "wpn_ak74",
  )
  .expect("expect an opted in section to describe an icon");

  assert_eq!(descriptor.x, 25);
  assert_eq!(descriptor.y, 4);
  assert_eq!(descriptor.w, 5);
  assert_eq!(descriptor.h, 2);
}

#[test]
fn ignores_grid_fields_without_opt_in() {
  assert!(
    descriptor_for(
      "[some_section]\n\
       inv_grid_x = 25\n\
       inv_grid_y = 4\n\
       inv_grid_width = 5\n\
       inv_grid_height = 2\n",
      "some_section",
    )
    .is_none(),
    "Expect grid fields alone not to declare an icon, so adding them cannot pack an asset"
  );
}

#[test]
fn ignores_sections_that_opt_out() {
  assert!(
    descriptor_for(
      "[af_base]\n\
       $inventory_icon = false\n\
       inv_grid_x = 0\n\
       inv_grid_y = 0\n\
       inv_grid_width = 1\n\
       inv_grid_height = 1\n",
      "af_base",
    )
    .is_none(),
    "Expect an explicit opt out to be honoured even when the section is grid complete"
  );
}

fn ltx_for(slots: &[(u32, u32, u32, u32)]) -> Ltx {
  let mut source: String = String::new();

  for (index, (x, y, w, h)) in slots.iter().enumerate() {
    source.push_str(&format!(
      "[section_{index}]\n\
       $inventory_icon = true\n\
       inv_grid_x = {x}\n\
       inv_grid_y = {y}\n\
       inv_grid_width = {w}\n\
       inv_grid_height = {h}\n\n"
    ));
  }

  Ltx::read_from_str(&source).expect("test LTX is valid")
}

fn boundaries_of(slots: &[(u32, u32, u32, u32)]) -> (u32, u32) {
  InventorySpriteDescriptor::get_equipment_sprite_boundaries_from_ltx(&ltx_for(slots))
}

fn pixels_of(slot: (u32, u32, u32, u32)) -> (u32, u32, u32, u32) {
  InventorySpriteDescriptor::new_list_from_ltx(&ltx_for(&[slot]))[0].get_boundaries()
}

fn sheet_for(slots: &[(u32, u32, u32, u32)]) -> XrfResult<RgbaImage> {
  InventorySpriteDescriptor::create_equipment_sprite_base_for_ltx(&ltx_for(slots))
}

#[test]
fn bounds_the_sheet_by_its_furthest_grid_slots() {
  // Slots reach 30 columns by 20 rows of 50 pixels, which is aligned already.
  assert_eq!(boundaries_of(&[(25, 4, 5, 2), (0, 17, 1, 3)]), (1500, 1000));
}

#[test]
fn rounds_an_odd_column_up_to_a_whole_block() {
  // An odd column or row ends on a 50 pixel boundary, which is two pixels into a block.
  assert_eq!(boundaries_of(&[(0, 0, 1, 1)]), (52, 52));
  assert_eq!(boundaries_of(&[(0, 0, 2, 2)]), (100, 100));
}

#[test]
fn saturates_a_column_whose_pixels_pass_u32() {
  // 85,899,346 columns of 50 pixels wrap a `u32` to 4, which would place the icon back inside the sheet.
  assert_eq!(boundaries_of(&[(85_899_346, 0, 1, 1)]), (u32::MAX, 52));
  assert_eq!(pixels_of((85_899_346, 0, 1, 1)), (u32::MAX, 0, 50, 50));
}

#[test]
fn saturates_a_row_whose_end_passes_u32() {
  assert_eq!(boundaries_of(&[(0, u32::MAX, 1, 1)]), (52, u32::MAX));
}

#[test]
fn refuses_a_sheet_whose_grid_passes_u32() {
  assert!(sheet_for(&[(85_899_346, 0, 1, 1)]).is_err());
  assert!(sheet_for(&[(0, u32::MAX, 1, 1)]).is_err());
}

#[test]
fn allocates_the_widest_sheet_under_the_size_limit() {
  // 655 columns end at 32,750 pixels, aligned to 32,752; a 656th passes 32K.
  let sheet: RgbaImage = sheet_for(&[(654, 0, 1, 1)]).expect("expect the widest sheet under 32K to allocate");

  assert_eq!(sheet.dimensions(), (32_752, 52));
  assert!(sheet_for(&[(655, 0, 1, 1)]).is_err());
}

#[test]
fn fits_a_slot_on_the_sheet_edge() {
  let (x, y, w, h) = pixels_of((654, 1, 1, 1));
  let sheet: RgbaImage = sheet_for(&[(654, 1, 1, 1)]).expect("expect an edge slot to allocate");

  assert_eq!((x + w, y + h), (32_750, 100));
  assert!(x + w <= sheet.width() && y + h <= sheet.height());
}

#[test]
fn requires_grid_fields_even_when_opted_in() {
  assert!(
    descriptor_for(
      "[af_base]\n\
       $inventory_icon = true\n\
       inv_grid_width = 1\n\
       inv_grid_height = 1\n",
      "af_base",
    )
    .is_none(),
    "Expect a section without grid position not to describe an icon"
  );
}
