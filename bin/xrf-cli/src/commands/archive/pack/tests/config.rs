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

#[test]
fn selects_the_same_files_named_directly_as_through_a_configuration() -> CommandResult {
  let (source, destination) = create_roots("direct")?;
  let config: PathBuf = source.parent().expect("a parent").join("pack.json");

  fs::write(
    &config,
    r#"{ "includeDirectories": [{ "path": "configs", "isRecursive": true }] }"#,
  )?;

  let through_file: u64 = pack_with_config(&source, &destination.join("file"), &config)?;

  let mut arguments: Vec<String> = pack_arguments(&source, &destination.join("direct"), "cfg");

  arguments.push(String::from("--include-directory"));
  arguments.push(String::from("configs"));

  let result: Value = run_command_for_result(&PackCommand, &arguments)?.expect("pack reports a result");
  let directly: u64 = result
    .get("filesTotal")
    .and_then(Value::as_u64)
    .expect("the result counts what was packed");

  assert_eq!(through_file, 1, "the configuration narrowed the selection");
  assert_eq!(directly, through_file, "naming it directly selects the same files");

  Ok(())
}

#[test]
fn refuses_a_configuration_file_beside_a_direct_selection() -> CommandResult {
  let (source, destination) = create_roots("conflict")?;
  let config: PathBuf = source.parent().expect("a parent").join("pack.json");

  fs::write(&config, r#"{ "includeFiles": [] }"#)?;

  let mut arguments: Vec<String> = pack_arguments(&source, &destination, "cfg");

  arguments.push(String::from("--config"));
  arguments.push(config.display().to_string());
  arguments.push(String::from("--include-directory"));
  arguments.push(String::from("configs"));

  // Two sources for one selection would need a precedence rule; clap refuses instead of inventing one.
  let error: String = run_command(&PackCommand, &arguments)
    .expect_err("a file beside a direct selection is refused")
    .to_string();

  assert!(error.contains("--config"), "{error}");
  assert!(error.contains("--include-directory"), "{error}");

  Ok(())
}

#[test]
fn writes_the_header_entries_named_on_the_command_line() -> CommandResult {
  let (source, destination) = create_roots("header")?;

  let mut arguments: Vec<String> = pack_arguments(&source, &destination, "cfg");

  // A value may hold its own `=` and its own quotes, which the engine's own header does.
  for entry in [
    "auto_load=true",
    "entry_point=$fs_root$\\gamedata\\",
    "creator=\"gsc game world\"",
  ] {
    arguments.push(String::from("--header"));
    arguments.push(String::from(entry));
  }

  arguments.push(String::from("--include-directory"));
  arguments.push(String::from("configs"));

  run_command(&PackCommand, &arguments)?;

  // Read back out of the volume the run published, which is where the header actually has to land.
  let volume: Vec<u8> = fs::read(destination.join("cfg.db"))?;
  let text: String = String::from_utf8_lossy(&volume).into_owned();

  for expected in [
    "auto_load = true",
    "entry_point = $fs_root$\\gamedata\\",
    "creator = \"gsc game world\"",
  ] {
    assert!(text.contains(expected), "the volume carries '{expected}'");
  }

  Ok(())
}

#[test]
fn refuses_a_header_entry_that_names_no_key() -> CommandResult {
  let (source, destination) = create_roots("header-invalid")?;

  for entry in ["auto_load", "=true"] {
    let mut arguments: Vec<String> = pack_arguments(&source, &destination, "cfg");

    arguments.push(String::from("--header"));
    arguments.push(String::from(entry));

    let error: String = run_command(&PackCommand, &arguments)
      .expect_err("a malformed header entry is refused")
      .to_string();

    assert!(error.contains(entry), "{error}");
  }

  Ok(())
}

#[test]
fn packs_the_whole_tree_when_nothing_is_selected() -> CommandResult {
  let (source, destination) = create_roots("bare")?;

  // Only a source: no configuration file and no selection option. Naming nothing means everything, as it does when
  // xrCompress is handed a directory and no LTX, so both files on disk are packed.
  let result: Value = run_command_for_result(&PackCommand, &pack_arguments(&source, &destination, "cfg"))?
    .expect("pack reports a result");

  assert_eq!(
    result.get("filesTotal").and_then(Value::as_u64),
    Some(2),
    "an unnarrowed run takes the whole source tree"
  );

  // The mountable default header survives a run that named no header of its own.
  let volume: Vec<u8> = fs::read(destination.join("cfg.db"))?;

  assert!(
    String::from_utf8_lossy(&volume).contains("entry_point = $fs_root$\\gamedata\\"),
    "the default header is still written"
  );

  Ok(())
}
