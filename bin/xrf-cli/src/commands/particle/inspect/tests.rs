use std::path::PathBuf;

use serde_json::Value;

use crate::commands::particle::fixtures::{create_library, run_for_result};
use crate::commands::particle::inspect::command::InspectCommand;
use crate::core::command_testing::run_command_for_result;
use crate::core::generic_command::CommandResult;

#[test]
fn deposits_an_effect_whole_found_in_any_case_and_with_either_slash() -> CommandResult {
  let library: PathBuf = create_library("inspect_effect")?;
  let result: Value = run_for_result(&InspectCommand, &library, &["FX/Flame"])?;

  assert_eq!(result["group"], Value::Null);
  assert_eq!(result["effect"]["name"], r"fx\flame");
  assert_eq!(result["effect"]["maxParticles"], 8);
  assert_eq!(result["effect"]["actions"].as_array().unwrap().len(), 3);
  assert_eq!(result["effect"]["frame"]["frameCount"], 8);

  Ok(())
}

#[test]
fn deposits_a_group_whole_with_its_effects_and_children() -> CommandResult {
  let library: PathBuf = create_library("inspect_group")?;
  let result: Value = run_for_result(&InspectCommand, &library, &[r"fx\campfire"])?;
  let effects: &Vec<Value> = result["group"]["effects"].as_array().unwrap();

  assert_eq!(result["effect"], Value::Null);
  assert_eq!(effects.len(), 2);
  assert_eq!(effects[1]["name"], r"fx\smoke");
  assert_eq!(effects[1]["onBirthChildName"], r"fx\haze");

  Ok(())
}

#[test]
fn answers_a_missing_name_with_the_names_holding_it() -> CommandResult {
  let library: PathBuf = create_library("inspect_missing")?;
  let arguments: Vec<String> = ["inspect", "--path", &library.display().to_string(), "smok"]
    .map(String::from)
    .to_vec();
  let error: String = run_command_for_result(&InspectCommand, &arguments)
    .expect_err("Expected a missing name to fail")
    .to_string();

  assert!(error.contains("No effect or group 'smok'"), "{error}");
  assert!(error.contains(r"names holding it: fx\smoke"), "{error}");

  Ok(())
}
