use xrf_error::XrfResult;
use xrf_ltx::{Ltx, LtxStandardDialect};
use xrf_test_utils::utils::{
  build_absolute_generated_test_resource_path, build_relative_test_sample_file_path, write_generated_test_resource,
};

use crate::equipment::{EquipmentSlotClaim, EquipmentSlotOccupant};

/// The reader is given a resolved config, which is what applies inheritance; a raw parse leaves a child holding only
/// the fields written under its own header.
fn resolved(source: &str) -> Ltx {
  Ltx::read_from_str(source)
    .expect("test LTX is valid")
    .into_inherited()
    .expect("test LTX resolves")
}

fn occupant_for(source: &str, section: &str) -> Option<EquipmentSlotOccupant> {
  let ltx: Ltx = resolved(source);

  EquipmentSlotOccupant::new_optional_from_section(section, &ltx[section])
}

#[test]
fn reads_a_section_that_only_positions_itself() {
  // Every shipped tree looks like this. Requiring the marker here is what showed an empty grid for vanilla Call of
  // Pripyat, Clear Sky, Call of Chernobyl and Anomaly alike.
  let occupant: EquipmentSlotOccupant = occupant_for(
    "[wpn_ak74]\ninv_grid_x = 25\ninv_grid_y = 4\ninv_grid_width = 5\ninv_grid_height = 2\n",
    "wpn_ak74",
  )
  .expect("expect grid fields alone to place a section");

  assert_eq!(occupant.claim, EquipmentSlotClaim::Probable);
  assert_eq!((occupant.x, occupant.y, occupant.w, occupant.h), (25, 4, 5, 2));
  assert_eq!(occupant.custom_icon, None);
  assert_eq!(occupant.origin, None);
}

#[test]
fn marks_a_section_the_packing_tools_act_on() {
  let occupant: EquipmentSlotOccupant = occupant_for(
    "[wpn_ak74]\n\
     $inventory_icon = true\n\
     $inventory_icon_path = ~\\textures\\ui\\ak74\n\
     inv_grid_x = 25\n\
     inv_grid_y = 4\n\
     inv_grid_width = 5\n\
     inv_grid_height = 2\n",
    "wpn_ak74",
  )
  .expect("expect a marked section to be read");

  assert_eq!(occupant.claim, EquipmentSlotClaim::Declared);
  assert_eq!(occupant.custom_icon.as_deref(), Some("~\\textures\\ui\\ak74"));
}

#[test]
fn honours_an_explicit_opt_out() {
  // Opting out of the packing tools is a statement about the icon. Reporting the section anyway, as a weaker claim,
  // would be the inference this type exists to avoid.
  assert!(
    occupant_for(
      "[af_base]\n$inventory_icon = false\ninv_grid_x = 0\ninv_grid_y = 0\ninv_grid_width = 1\ninv_grid_height = 1\n",
      "af_base",
    )
    .is_none(),
    "Expect an explicit opt out to be honoured even when the section is grid complete"
  );
}

#[test]
fn requires_every_grid_field() {
  assert!(
    occupant_for("[af_base]\ninv_grid_width = 1\ninv_grid_height = 1\n", "af_base").is_none(),
    "Expect a section without a grid position to occupy nothing"
  );
}

#[test]
fn refuses_an_icon_that_covers_no_cell() {
  assert!(
    occupant_for(
      "[af_base]\ninv_grid_x = 1\ninv_grid_y = 1\ninv_grid_width = 0\ninv_grid_height = 1\n",
      "af_base",
    )
    .is_none(),
    "Expect a zero-width icon to occupy nothing, since it would be drawn nowhere"
  );
}

#[test]
fn refuses_a_position_that_is_not_a_whole_number_of_cells() {
  assert!(
    occupant_for(
      "[af_base]\ninv_grid_x = 1.5\ninv_grid_y = 1\ninv_grid_width = 1\ninv_grid_height = 1\n",
      "af_base",
    )
    .is_none(),
    "Expect a position the grid cannot express to occupy nothing"
  );
}

#[test]
fn reads_an_inherited_position_at_the_slot_the_engine_draws() {
  // Upgrade, `_nimble` and `_snag` variants inherit their base weapon's position and share its slot. Resolved
  // lookups see inherited fields, which is what makes them report there too.
  let ltx: Ltx = resolved(
    "[wpn_ak74]\n\
     inv_grid_x = 25\n\
     inv_grid_y = 4\n\
     inv_grid_width = 5\n\
     inv_grid_height = 2\n\
     \n\
     [wpn_ak74_nimble]:wpn_ak74\n\
     cost = 10000\n",
  );

  let occupants: Vec<EquipmentSlotOccupant> = EquipmentSlotOccupant::new_list_from_ltx(&ltx);

  assert_eq!(occupants.len(), 2);
  // Declaration order, so a cell can name every section drawn there rather than only the last to claim it.
  assert_eq!(occupants[0].section, "wpn_ak74");
  assert_eq!(occupants[1].section, "wpn_ak74_nimble");
  assert_eq!((occupants[1].x, occupants[1].y), (25, 4));
}

#[test]
fn inherits_the_marker_the_packing_tools_read() {
  // A child of a marked section is packed today, so it has to be reported the same way here or the two answers
  // disagree about the same section.
  let ltx: Ltx = resolved(
    "[wpn_ak74]\n\
     $inventory_icon = true\n\
     inv_grid_x = 25\n\
     inv_grid_y = 4\n\
     inv_grid_width = 5\n\
     inv_grid_height = 2\n\
     \n\
     [wpn_ak74_nimble]:wpn_ak74\n\
     cost = 10000\n",
  );

  let occupants: Vec<EquipmentSlotOccupant> = EquipmentSlotOccupant::new_list_from_ltx(&ltx);

  assert_eq!(occupants[1].claim, EquipmentSlotClaim::Declared);
}

#[test]
fn skips_sections_that_occupy_nothing() {
  let ltx: Ltx = resolved(
    "[some_settings]\n\
     value = 1\n\
     \n\
     [wpn_ak74]\n\
     inv_grid_x = 0\n\
     inv_grid_y = 0\n\
     inv_grid_width = 1\n\
     inv_grid_height = 1\n",
  );

  let occupants: Vec<EquipmentSlotOccupant> = EquipmentSlotOccupant::new_list_from_ltx(&ltx);

  assert_eq!(occupants.len(), 1);
  assert_eq!(occupants[0].section, "wpn_ak74");
}

#[test]
fn records_the_config_a_section_was_declared_in() -> XrfResult {
  // Read through a dialect rather than from a string or through `read_from_file_standard`: the origin is stamped as
  // the dialect resolves each document, and neither of the other two doors goes through one. This is the door the
  // sprite plugin uses.
  let path: String = build_relative_test_sample_file_path(file!(), "declaring_config.ltx");

  write_generated_test_resource(
    &path,
    b"[wpn_ak74]
inv_grid_x = 0
inv_grid_y = 0
inv_grid_width = 1
inv_grid_height = 1
",
  )?;

  let ltx: Ltx =
    Ltx::read_from_file_with_dialect(build_absolute_generated_test_resource_path(&path), &LtxStandardDialect)?;
  let occupants: Vec<EquipmentSlotOccupant> = EquipmentSlotOccupant::new_list_from_ltx(&ltx);

  assert_eq!(occupants.len(), 1);
  // The dialect stamps whatever it resolved the document by, which for a bare file is its path and for a mounted
  // root is the logical path inside it. Both name the declaring config; only the spelling differs.
  assert!(
    occupants[0]
      .origin
      .as_deref()
      .is_some_and(|origin| origin.ends_with("declaring_config.ltx")),
    "Expect the occupant to name the config its section was declared in, got {:?}",
    occupants[0].origin
  );

  Ok(())
}
