//! Exercises how `archive pack` reads `--config`: which codec it selects, and what an explicit option still wins over.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::commands::archive::pack::command::PackCommand;
use crate::core::command_testing::{run_command, run_command_for_result};
use crate::core::generic_command::CommandResult;

/// A source tree of two directories, so a configuration that selects one proves it selected.
fn create_roots(name: &str) -> CommandResult<(PathBuf, PathBuf)> {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("archive_pack_config/{name}"));

  if root.exists() {
    fs::remove_dir_all(&root)?;
  }

  fs::create_dir_all(root.join("source").join("configs"))?;
  fs::create_dir_all(root.join("source").join("meshes"))?;
  fs::write(
    root.join("source").join("configs").join("system.ltx"),
    "[section]\nvalue = 1\n",
  )?;
  fs::write(root.join("source").join("meshes").join("actor.ogf"), "visual")?;

  Ok((root.join("source"), root.join("packed")))
}

fn pack_arguments(source: &Path, destination: &Path, name: &str) -> Vec<String> {
  vec![
    String::from("pack"),
    String::from("--path"),
    source.display().to_string(),
    String::from("--dest"),
    destination.display().to_string(),
    String::from("--name"),
    String::from(name),
    String::from("--silent"),
    String::from("--json"),
  ]
}

/// Pack through a configuration file and answer how many files it selected.
fn pack_with_config(source: &Path, destination: &Path, config: &Path) -> CommandResult<u64> {
  let mut arguments: Vec<String> = pack_arguments(source, destination, "cfg");

  arguments.push(String::from("--config"));
  arguments.push(config.display().to_string());

  let result: Value = run_command_for_result(&PackCommand, &arguments)?.expect("pack reports a result");

  Ok(
    result
      .get("filesTotal")
      .and_then(Value::as_u64)
      .expect("the result counts what was packed"),
  )
}

#[test]
fn reads_both_serializations_of_one_configuration() -> CommandResult {
  let (source, destination) = create_roots("equivalent")?;
  let root: &Path = source.parent().expect("a parent");

  // Both select the configs directory alone, so both must pack one file out of the two on disk.
  let ltx: PathBuf = root.join("pack.ltx");
  let json: PathBuf = root.join("pack.json");

  fs::write(&ltx, "[include_folders]\nconfigs = true\n")?;
  fs::write(
    &json,
    r#"{ "includeDirectories": [{ "path": "configs", "isRecursive": true }] }"#,
  )?;

  let from_ltx: u64 = pack_with_config(&source, &destination.join("ltx"), &ltx)?;
  let from_json: u64 = pack_with_config(&source, &destination.join("json"), &json)?;

  assert_eq!(from_ltx, 1, "the configuration narrowed the selection");
  assert_eq!(from_json, from_ltx, "the two serializations select the same files");

  Ok(())
}

#[test]
fn refuses_a_configuration_whose_extension_names_no_format() -> CommandResult {
  let (source, destination) = create_roots("unsupported")?;
  let config: PathBuf = source.parent().expect("a parent").join("pack.txt");

  // The contents are a perfectly good LTX; the name is what is refused, because guessing would let a mistyped path
  // succeed against the wrong reader.
  fs::write(&config, "[include_folders]\nconfigs = true\n")?;

  let mut arguments: Vec<String> = pack_arguments(&source, &destination, "cfg");

  arguments.push(String::from("--config"));
  arguments.push(config.display().to_string());

  let error: String = run_command(&PackCommand, &arguments)
    .expect_err("an unsupported extension is refused")
    .to_string();

  assert!(error.contains(".ltx"), "{error}");
  assert!(error.contains(".json"), "{error}");

  Ok(())
}

#[test]
fn keeps_an_explicit_option_winning_over_an_imported_one() -> CommandResult {
  let (source, destination) = create_roots("precedence")?;
  let config: PathBuf = source.parent().expect("a parent").join("pack.json");

  // The file owns selection rules and the header; the volume name and the extension are the invocation's, and a
  // configuration file carries neither, so naming them on the command line is the only thing that decides them.
  fs::write(
    &config,
    r#"{
      "includeDirectories": [{ "path": "configs", "isRecursive": true }],
      "header": [{ "key": "entry_point", "value": "$fs_root$\\levels\\" }]
    }"#,
  )?;

  let mut arguments: Vec<String> = pack_arguments(&source, &destination, "chosen");

  arguments.push(String::from("--config"));
  arguments.push(config.display().to_string());
  arguments.push(String::from("--xdb"));

  run_command(&PackCommand, &arguments)?;

  assert!(
    destination.join("chosen.xdb").exists(),
    "the command line named the volume, not the configuration file: {:?}",
    fs::read_dir(&destination)?
      .filter_map(Result::ok)
      .map(|entry| entry.file_name())
      .collect::<Vec<_>>()
  );

  Ok(())
}
