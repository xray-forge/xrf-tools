use std::path::PathBuf;

use serde_json::{Value, json};

use crate::commands::particle::fixtures::{create_library, run_for_result};
use crate::commands::particle::list::command::ListCommand;
use crate::core::generic_command::CommandResult;

#[test]
fn lists_every_effect_then_every_group_with_what_tells_each_apart() -> CommandResult {
  let library: PathBuf = create_library("list_all")?;
  let result: Value = run_for_result(&ListCommand, &library, &[])?;
  let entries: &Vec<Value> = result["entries"].as_array().unwrap();
  let names: Vec<&str> = entries.iter().map(|it| it["name"].as_str().unwrap()).collect();

  // In the library's order, which packing from the unpacked files makes alphabetical.
  assert_eq!(
    names,
    [r"fx\flame", r"fx\haze", r"fx\smoke", r"fx\campfire", r"fx\puff"]
  );
  assert_eq!(
    entries[0],
    json!({
      "kind": "effect",
      "name": r"fx\flame",
      "shader": r"particles\xadd",
      // The list split as the engine splits it, each name kept as the library spells it.
      "textures": [r"pfx\pfx_flame.bmp", r"pfx\pfx_distortion"],
      "maxParticles": 8,
      "actions": ["KillOld", "TargetColor", "Move"],
      "timeLimit": 2.0,
    })
  );
  assert_eq!(entries[1]["timeLimit"], Value::Null);
  assert_eq!(
    entries[3],
    json!({
      "kind": "group",
      "name": r"fx\campfire",
      "effects": [r"fx\flame", r"fx\smoke"],
      "children": [r"fx\haze"],
      "timeLimit": 0.0,
    })
  );

  Ok(())
}

#[test]
fn keeps_one_kind_and_names_holding_a_text_in_any_case() -> CommandResult {
  let library: PathBuf = create_library("list_filtered")?;
  let groups: Value = run_for_result(&ListCommand, &library, &["--kind", "group"])?;
  let hazes: Value = run_for_result(&ListCommand, &library, &["--kind", "effect", "--name", "AZE"])?;

  assert_eq!(groups["entries"].as_array().unwrap().len(), 2);
  assert!(
    groups["entries"]
      .as_array()
      .unwrap()
      .iter()
      .all(|it| it["kind"] == "group")
  );
  assert_eq!(hazes["entries"].as_array().unwrap().len(), 1);
  assert_eq!(hazes["entries"][0]["name"], r"fx\haze");

  Ok(())
}
