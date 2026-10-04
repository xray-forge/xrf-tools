use std::fs;
use std::path::PathBuf;

use serde_json::{Value, json};

use crate::commands::dialog::find::command::FindCommand;
use crate::commands::dialog::fixtures::{create_tree, run_for_result};
use crate::core::command_testing::run_command_for_result;
use crate::core::generic_command::CommandResult;

/// A dialog whose entry phrase carries a condition nothing ever gives, as Cardan's first drink does.
const DRINKS: &str = r#"<game_dialogs>
  <dialog id="tech_drink_dialog">
    <precondition>dialogs.if_actor_has_vodka</precondition>
    <phrase_list>
      <phrase id="0"><text>drink_0</text><has_info>tech_drink_first_time</has_info><next>1</next></phrase>
      <phrase id="1"><text>drink_1</text><action>dialogs_zaton.give_vodka</action><give_info>snag_cache_known</give_info></phrase>
    </phrase_list>
  </dialog>
</game_dialogs>
"#;

fn create_drinks_tree(name: &str) -> CommandResult<PathBuf> {
  let root: PathBuf = create_tree(name)?;

  fs::write(root.join("configs").join("gameplay").join("dialogs_drinks.xml"), DRINKS)?;

  Ok(root)
}

/// Each reference as `(dialog, phrase, element, value, ignored)`, which is what a search is judged on.
fn summarise(result: &Value) -> Vec<(String, Value, String, String, bool)> {
  result["references"]
    .as_array()
    .expect("Expected references")
    .iter()
    .map(|reference| {
      (
        reference["dialogId"].as_str().unwrap_or_default().to_owned(),
        reference["phraseId"].clone(),
        reference["element"]["name"].as_str().unwrap_or_default().to_owned(),
        reference["element"]["value"].as_str().unwrap_or_default().to_owned(),
        reference["isIgnored"].as_bool().unwrap_or_default(),
      )
    })
    .collect()
}

#[test]
fn finds_where_dialogs_check_give_and_take_an_info_portion() -> CommandResult {
  let root: PathBuf = create_drinks_tree("find_info")?;
  let result: Value = run_for_result(&FindCommand, &root, &["--info", "snag_cache_known"])?;

  assert_eq!(result["infos"], json!(["snag_cache_known"]));
  assert_eq!(result["functors"], json!([]));
  assert_eq!(
    summarise(&result),
    vec![
      (
        "snag_cache_dialog".into(),
        Value::Null,
        "dont_has_info".into(),
        "snag_cache_known".into(),
        false
      ),
      (
        "snag_cache_dialog".into(),
        json!("1"),
        "give_info".into(),
        "snag_cache_known".into(),
        false
      ),
      (
        "snag_share_dialog".into(),
        Value::Null,
        "has_info".into(),
        "snag_cache_known".into(),
        false
      ),
      (
        "tech_drink_dialog".into(),
        json!("1"),
        "give_info".into(),
        "snag_cache_known".into(),
        false
      ),
    ]
  );
  assert_eq!(result["references"][0]["logicalPath"], r"configs\gameplay\dialogs.xml");
  assert_eq!(result["references"][0]["element"]["kind"], "dontHasInfo");

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn marks_an_entry_phrase_condition_the_engine_never_reads() -> CommandResult {
  let root: PathBuf = create_drinks_tree("find_ignored")?;
  let result: Value = run_for_result(&FindCommand, &root, &["--info", "tech_drink_first_time"])?;

  assert_eq!(
    summarise(&result),
    vec![(
      "tech_drink_dialog".into(),
      json!("0"),
      "has_info".into(),
      "tech_drink_first_time".into(),
      true
    )]
  );

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn finds_a_function_by_its_full_name_or_by_its_name_in_any_module() -> CommandResult {
  let root: PathBuf = create_drinks_tree("find_functor")?;
  let result: Value = run_for_result(
    &FindCommand,
    &root,
    &[
      "--functor",
      "give_task",
      "--functor",
      "dialogs_zaton.give_vodka",
      "--functor",
      "dialogs.give_vodka",
    ],
  )?;

  assert_eq!(
    summarise(&result),
    vec![
      (
        "snag_cache_dialog".into(),
        json!("1"),
        "action".into(),
        "dialogs.give_task".into(),
        false
      ),
      (
        "tech_drink_dialog".into(),
        json!("1"),
        "action".into(),
        "dialogs_zaton.give_vodka".into(),
        false
      ),
    ]
  );

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn answers_a_name_nothing_references_with_no_references() -> CommandResult {
  let root: PathBuf = create_drinks_tree("find_nothing")?;
  let result: Value = run_for_result(&FindCommand, &root, &["--info", "nobody_gives_this"])?;

  assert_eq!(result["references"], json!([]));

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn refuses_a_search_naming_nothing_to_find() -> CommandResult {
  let root: PathBuf = create_drinks_tree("find_no_query")?;
  let arguments: Vec<String> = vec![
    String::from("find"),
    String::from("--path"),
    root.display().to_string(),
    String::from("--silent"),
  ];

  assert!(run_command_for_result(&FindCommand, &arguments).is_err());

  fs::remove_dir_all(root)?;

  Ok(())
}
