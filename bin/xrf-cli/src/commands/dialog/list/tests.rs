use std::fs;
use std::path::PathBuf;

use serde_json::{Value, json};

use crate::commands::dialog::fixtures::{create_tree, run_for_result};
use crate::commands::dialog::list::command::ListCommand;
use crate::core::generic_command::CommandResult;

#[test]
fn lists_every_dialog_with_its_caption_and_conditions() -> CommandResult {
  let root: PathBuf = create_tree("list_all")?;
  let result: Value = run_for_result(&ListCommand, &root, &[])?;

  assert_eq!(result["language"], "eng");
  assert_eq!(result["profile"], Value::Null);

  let dialogs: &Vec<Value> = result["dialogs"].as_array().unwrap();

  assert_eq!(dialogs.len(), 4);
  assert_eq!(dialogs[0]["id"], "snag_cache_dialog");
  assert_eq!(dialogs[0]["logicalPath"], r"configs\gameplay\dialogs.xml");
  assert_eq!(dialogs[0]["phrases"], 2);
  assert_eq!(dialogs[0]["captionKey"], "snag_cache_0");
  assert_eq!(dialogs[0]["caption"], "Want to earn some?");
  assert_eq!(
    dialogs[0]["elements"],
    json!([{ "name": "dont_has_info", "kind": "dontHasInfo", "value": "snag_cache_known" }])
  );
  assert_eq!(dialogs[0]["offers"], json!([]));
  assert_eq!(dialogs[1]["priority"], 5);
  // Untranslated keys stay keys, and the overlay's copy of a dialog is listed as the file it is in.
  assert_eq!(dialogs[2]["caption"], Value::Null);
  assert_eq!(dialogs[3]["logicalPath"], r"configs\gameplay\dialogs_overlay.xml");

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn lists_the_dialogs_reaching_the_actor_from_a_profile_with_each_way_they_do() -> CommandResult {
  let root: PathBuf = create_tree("list_profile")?;
  let result: Value = run_for_result(&ListCommand, &root, &["--profile", "snag"])?;

  assert_eq!(result["profile"], json!({ "id": "snag", "characters": ["snag"] }));

  let dialogs: &Vec<Value> = result["dialogs"].as_array().unwrap();
  let ids: Vec<&str> = dialogs.iter().map(|dialog| dialog["id"].as_str().unwrap()).collect();

  assert_eq!(ids, ["snag_cache_dialog", "snag_share_dialog", "actor_break_dialog"]);
  assert_eq!(dialogs[0]["offers"], json!([{ "kind": "start", "character": "snag" }]));
  assert_eq!(dialogs[1]["offers"], json!([{ "kind": "actor", "character": "snag" }]));
  assert_eq!(
    dialogs[2]["offers"],
    json!([{ "kind": "info", "info": "global_dialogs" }])
  );
  // The engine reads the first declaration of an id, which is not the overlay's.
  assert_eq!(dialogs[2]["logicalPath"], r"configs\gameplay\dialogs.xml");

  let findings: &Vec<Value> = result["findings"].as_array().unwrap();

  assert_eq!(findings.len(), 1);
  assert_eq!(findings[0]["rule"], "dialog.offer-missing");
  assert_eq!(findings[0]["subject"], "snag_missing_dialog");

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn refuses_a_profile_the_tree_does_not_declare() -> CommandResult {
  let root: PathBuf = create_tree("list_unknown_profile")?;
  let error: String = run_for_result(&ListCommand, &root, &["--profile", "nobody"])
    .unwrap_err()
    .message();

  assert!(error.contains("No profile 'nobody'"), "{error}");

  fs::remove_dir_all(root)?;

  Ok(())
}
