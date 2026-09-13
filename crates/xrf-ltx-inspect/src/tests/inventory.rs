//! What each config is to its project, and which resolution judges it.

use crate::inventory::{LtxInventory, LtxInventoryFile, LtxInventoryRole};

/// An inventory over configs held in memory, without a mounted world.
fn inventory_of(files: &[(&str, LtxInventoryRole, &[&str])]) -> LtxInventory {
  LtxInventory {
    files: files
      .iter()
      .map(|(path, role, included_by)| LtxInventoryFile {
        included_by: included_by.iter().map(|it| String::from(*it)).collect(),
        is_physical: true,
        path: String::from(*path),
        role: role.clone(),
        source: String::from("test"),
      })
      .collect(),
  }
}

#[test]
fn a_scheme_declaration_another_config_includes_resolves_through_that_config() {
  let inventory: LtxInventory = inventory_of(&[
    ("configs\\$scheme\\scheme.ltx", LtxInventoryRole::SchemeFile, &[]),
    (
      "configs\\$scheme\\items.scheme.ltx",
      LtxInventoryRole::SchemeFile,
      &["configs\\$scheme\\scheme.ltx"],
    ),
  ]);

  assert_eq!(
    inventory.list_entry_points_of("configs\\$scheme\\items.scheme.ltx"),
    vec![String::from("configs\\$scheme\\scheme.ltx")]
  );
  assert_eq!(
    inventory.list_entry_points_of("configs\\$scheme\\scheme.ltx"),
    vec![String::from("configs\\$scheme\\scheme.ltx")],
    "the declaration nothing includes to answer itself"
  );
}

#[test]
fn an_included_config_answers_every_entry_point_above_it_once() {
  // A diamond: both roots reach `w_base.ltx`, and the one they share is named once however many paths lead to it.
  let inventory: LtxInventory = inventory_of(&[
    ("system.ltx", LtxInventoryRole::EntryPoint, &[]),
    ("mp.ltx", LtxInventoryRole::EntryPoint, &[]),
    ("items.ltx", LtxInventoryRole::Included, &["system.ltx", "mp.ltx"]),
    ("w_base.ltx", LtxInventoryRole::Included, &["items.ltx"]),
  ]);

  assert_eq!(
    inventory.list_entry_points_of("w_base.ltx"),
    vec![String::from("mp.ltx"), String::from("system.ltx")]
  );
}

#[test]
fn an_attachment_answers_no_entry_point_at_all() {
  // A patch file belongs to the config it patches, and the dialect's plan does not record which that is. Answering
  // itself would resolve a file of overrides as a root and report every one of them as patching nothing.
  let inventory: LtxInventory = inventory_of(&[
    ("system.ltx", LtxInventoryRole::EntryPoint, &[]),
    ("mod_system_a.ltx", LtxInventoryRole::Attachment, &[]),
  ]);

  assert!(inventory.list_entry_points_of("mod_system_a.ltx").is_empty());
}

#[test]
fn a_config_the_project_does_not_hold_answers_nothing() {
  let inventory: LtxInventory = inventory_of(&[("system.ltx", LtxInventoryRole::EntryPoint, &[])]);

  assert!(inventory.list_entry_points_of("absent.ltx").is_empty());
}

#[test]
fn an_include_cycle_is_walked_once_rather_than_forever() {
  let inventory: LtxInventory = inventory_of(&[
    ("first.ltx", LtxInventoryRole::Included, &["second.ltx"]),
    ("second.ltx", LtxInventoryRole::Included, &["first.ltx"]),
  ]);

  // Neither reaches an entry point, and the answer is that rather than a hang.
  assert!(inventory.list_entry_points_of("first.ltx").is_empty());
}
