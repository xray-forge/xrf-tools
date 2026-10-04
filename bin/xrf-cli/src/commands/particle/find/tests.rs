use std::path::PathBuf;

use serde_json::{Value, json};

use crate::commands::particle::find::command::FindCommand;
use crate::commands::particle::fixtures::{create_library, run_for_result};
use crate::core::command_testing::run_command_for_result;
use crate::core::generic_command::CommandResult;

/// Each match's name and reasons, in order.
fn to_found(result: &Value) -> Vec<(String, Value)> {
  result["matches"]
    .as_array()
    .unwrap()
    .iter()
    .map(|it| (it["name"].as_str().unwrap().to_owned(), it["reasons"].clone()))
    .collect()
}

#[test]
fn finds_effects_by_texture_in_any_case_and_extension_and_by_shader() -> CommandResult {
  let library: PathBuf = create_library("find_sprite")?;
  let result: Value = run_for_result(
    &FindCommand,
    &library,
    &["--texture", "PFX/pfx_flame", "--shader", r"PARTICLES\blend"],
  )?;

  assert_eq!(result["textures"], json!(["PFX/pfx_flame"]));
  assert_eq!(
    to_found(&result),
    [
      (
        String::from(r"fx\flame"),
        json!([{ "kind": "texture", "value": r"pfx\pfx_flame.bmp" }])
      ),
      (
        String::from(r"fx\smoke"),
        json!([{ "kind": "shader", "value": r"particles\blend" }])
      ),
    ]
  );
  // A match carries its entry as `particle list` describes it.
  assert_eq!(result["matches"][0]["kind"], "effect");
  assert_eq!(result["matches"][0]["maxParticles"], 8);

  Ok(())
}

#[test]
fn finds_the_effects_running_an_action() -> CommandResult {
  let library: PathBuf = create_library("find_action")?;
  let result: Value = run_for_result(&FindCommand, &library, &["--action", "TargetColor"])?;

  assert_eq!(
    to_found(&result),
    [(
      String::from(r"fx\flame"),
      json!([{ "kind": "action", "value": "TargetColor" }])
    )]
  );

  Ok(())
}

#[test]
fn finds_the_groups_playing_an_effect_or_spawning_it_as_a_child() -> CommandResult {
  let library: PathBuf = create_library("find_effect")?;
  let result: Value = run_for_result(
    &FindCommand,
    &library,
    &["--effect", r"fx\smoke", "--effect", r"fx\haze"],
  )?;

  assert_eq!(
    to_found(&result),
    [
      (
        String::from(r"fx\campfire"),
        json!([
          { "kind": "effect", "value": r"fx\smoke" },
          { "kind": "birthChild", "value": r"fx\haze" },
        ])
      ),
      (
        String::from(r"fx\puff"),
        json!([{ "kind": "effect", "value": r"fx\smoke" }])
      ),
    ]
  );

  Ok(())
}

#[test]
fn refuses_an_action_type_the_library_format_has_not() -> CommandResult {
  let library: PathBuf = create_library("find_unknown_action")?;
  let arguments: Vec<String> = ["find", "--path", &library.display().to_string(), "--action", "Flicker"]
    .map(String::from)
    .to_vec();

  assert!(run_command_for_result(&FindCommand, &arguments).is_err());

  Ok(())
}
