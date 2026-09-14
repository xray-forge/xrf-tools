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

fn boundaries_of(slots: &[(u32, u32, u32, u32)]) -> (u32, u32) {
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

  InventorySpriteDescriptor::get_equipment_sprite_boundaries_from_ltx(
    &Ltx::read_from_str(&source).expect("test LTX is valid"),
  )
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
