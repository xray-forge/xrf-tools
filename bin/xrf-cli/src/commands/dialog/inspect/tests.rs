use std::fs;
use std::path::PathBuf;

use serde_json::{Value, json};

use crate::commands::dialog::fixtures::{create_tree, run_for_result};
use crate::commands::dialog::inspect::command::InspectCommand;
use crate::core::generic_command::CommandResult;

#[test]
fn explains_a_dialog_with_its_conditions_and_every_phrase() -> CommandResult {
  let root: PathBuf = create_tree("inspect_dialog")?;
  let result: Value = run_for_result(&InspectCommand, &root, &["snag_cache_dialog"])?;
  let dialog: &Value = &result["dialog"];

  assert_eq!(dialog["id"], "snag_cache_dialog");
  assert_eq!(dialog["logicalPath"], r"configs\gameplay\dialogs.xml");
  assert_eq!(dialog["language"], "eng");
  assert_eq!(dialog["elements"][0]["name"], "dont_has_info");
  assert_eq!(dialog["phrases"][0]["text"], "Want to earn some?");
  assert_eq!(dialog["phrases"][0]["next"], json!(["1"]));
  assert_eq!(dialog["phrases"][1]["textKey"], "snag_cache_1");
  assert_eq!(
    dialog["phrases"][1]["elements"],
    json!([
      { "name": "text", "kind": "text", "value": "snag_cache_1" },
      { "name": "give_info", "kind": "giveInfo", "value": "snag_cache_known" },
      { "name": "action", "kind": "action", "value": "dialogs.give_task" }
    ])
  );
  assert_eq!(result["alsoDeclaredIn"], json!([]));

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn reads_the_first_declaration_and_names_the_others_or_the_one_file_asked_for() -> CommandResult {
  let root: PathBuf = create_tree("inspect_overlay")?;

  let first: Value = run_for_result(&InspectCommand, &root, &["actor_break_dialog"])?;

  assert_eq!(first["dialog"]["logicalPath"], r"configs\gameplay\dialogs.xml");
  assert_eq!(
    first["alsoDeclaredIn"],
    json!([r"configs\gameplay\dialogs_overlay.xml"])
  );

  let asked: Value = run_for_result(
    &InspectCommand,
    &root,
    &["actor_break_dialog", "--file", r"configs\gameplay\DIALOGS_OVERLAY.xml"],
  )?;

  assert_eq!(asked["dialog"]["logicalPath"], r"configs\gameplay\dialogs_overlay.xml");
  assert_eq!(asked["dialog"]["phrases"][0]["textKey"], "overlay_0");
  assert_eq!(asked["alsoDeclaredIn"], json!([r"configs\gameplay\dialogs.xml"]));

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn refuses_a_dialog_no_file_declares_or_a_file_that_does_not_declare_it() -> CommandResult {
  let root: PathBuf = create_tree("inspect_missing")?;

  let missing: String = run_for_result(&InspectCommand, &root, &["nothing"])
    .unwrap_err()
    .message();
  let elsewhere: String = run_for_result(
    &InspectCommand,
    &root,
    &["snag_cache_dialog", "--file", r"configs\gameplay\dialogs_overlay.xml"],
  )
  .unwrap_err()
  .message();

  assert!(missing.contains("declares dialog 'nothing'"), "{missing}");
  assert!(elsewhere.contains("is not declared in"), "{elsewhere}");

  fs::remove_dir_all(root)?;

  Ok(())
}
