//! What `ogf fix` writes, reports and refuses.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use xrf_error::XrfResult;

use super::fixtures;
use crate::commands::ogf::fix::command::FixCommand;
use crate::core::command_testing::run_command_with_result;
use crate::core::generic_command::CommandResult;
use crate::core::staged_write::faults::fail_next_staged_write;

/// Runs the command with a result requested, the way a caller reading the report would.
fn fix(arguments: &[&str]) -> (CommandResult, Option<Value>) {
  let mut all: Vec<String> = vec![String::from("fix")];

  all.extend(arguments.iter().map(|it| String::from(*it)));
  all.push(String::from("--silent"));
  all.push(String::from("--json"));

  run_command_with_result(&FixCommand, &all)
}

fn argument(path: &Path) -> String {
  path.display().to_string()
}

fn names_in(root: &Path) -> XrfResult<Vec<String>> {
  let mut names: Vec<String> = fs::read_dir(root)?
    .filter_map(Result::ok)
    .map(|entry| entry.file_name().to_string_lossy().to_string())
    .collect();

  names.sort();

  Ok(names)
}

#[test]
fn normalizes_a_split_visual_in_place_and_names_what_it_dropped() -> XrfResult {
  let root: PathBuf = fixtures::create_root("in-place")?;
  let file: PathBuf = root.join("split.ogf");

  fs::write(&file, fixtures::split_motion_ref()?)?;

  let (verdict, result) = fix(&["--path", &argument(&file)]);
  let result: Value = result.expect("Expect a result to be deposited");

  verdict.expect("Expect the visual to be fixed");

  assert_eq!(fs::read(&file)?, fixtures::well_formed()?);
  assert_eq!(result["checked"], 1);
  assert_eq!(result["normalized"], 1);
  assert_eq!(result["unchanged"], 0);
  assert_eq!(result["failed"], 0);
  assert_eq!(result["discardedSize"], fixtures::SPLIT_SIZE);
  assert_eq!(result["isDryRun"], false);
  assert_eq!(result["findings"], Value::Array(Vec::new()));

  let entry: &Value = &result["files"][0];

  assert_eq!(entry["cause"], "split-motion-ref");
  assert_eq!(entry["discardedReference"], fixtures::SPLIT_REF);
  assert_eq!(entry["discardedSize"], fixtures::SPLIT_SIZE);
  assert_eq!(entry["isWritten"], true);
  assert_eq!(entry["originalSize"], fixtures::split_motion_ref()?.len());
  assert_eq!(entry["normalizedSize"], fixtures::well_formed()?.len());
  assert_eq!(entry["source"], entry["destination"]);
  assert_eq!(names_in(&root)?, vec![String::from("split.ogf")]);

  Ok(())
}

#[test]
fn leaves_a_visual_it_already_normalized_alone() -> XrfResult {
  let root: PathBuf = fixtures::create_root("idempotent")?;
  let file: PathBuf = root.join("split.ogf");

  fs::write(&file, fixtures::split_motion_ref()?)?;

  fix(&["--path", &argument(&file)])
    .0
    .expect("Expect the first run to fix the visual");

  let (verdict, result) = fix(&["--path", &argument(&file)]);
  let result: Value = result.expect("Expect a result to be deposited");

  verdict.expect("Expect the second run to succeed");

  assert_eq!(fs::read(&file)?, fixtures::well_formed()?);
  assert_eq!(result["checked"], 1);
  assert_eq!(result["normalized"], 0);
  assert_eq!(result["unchanged"], 1);
  assert_eq!(result["discardedSize"], 0);
  assert_eq!(result["files"], Value::Array(Vec::new()));

  Ok(())
}

#[test]
fn reports_a_dry_run_without_writing() -> XrfResult {
  let root: PathBuf = fixtures::create_root("dry-run")?;
  let file: PathBuf = root.join("split.ogf");

  fs::write(&file, fixtures::split_motion_ref()?)?;

  let (verdict, result) = fix(&["--path", &argument(&file), "--dry-run"]);
  let result: Value = result.expect("Expect a result to be deposited");

  verdict.expect("Expect a dry run to succeed");

  assert_eq!(fs::read(&file)?, fixtures::split_motion_ref()?);
  assert_eq!(result["isDryRun"], true);
  assert_eq!(result["normalized"], 1);
  assert_eq!(result["discardedSize"], fixtures::SPLIT_SIZE);
  assert_eq!(result["files"][0]["isWritten"], false);
  assert_eq!(result["files"][0]["discardedReference"], fixtures::SPLIT_REF);
  assert_eq!(names_in(&root)?, vec![String::from("split.ogf")]);

  Ok(())
}

/// A sweep fixes what it can and fails at the end for what it could not, rather than stopping at the first refusal.
#[test]
fn sweeps_a_directory_and_fails_for_what_it_refused() -> XrfResult {
  let root: PathBuf = fixtures::create_root("directory")?;
  let nested: PathBuf = root.join("nested");

  fs::create_dir_all(&nested)?;
  fs::write(root.join("clean.ogf"), fixtures::well_formed()?)?;
  fs::write(nested.join("split.OGF"), fixtures::split_motion_ref()?)?;
  fs::write(root.join("unexplained.ogf"), fixtures::unexplained_residue()?)?;
  fs::write(root.join("notes.txt"), b"not a visual")?;

  let (verdict, result) = fix(&["--path", &argument(&root)]);
  let result: Value = result.expect("Expect a result to be deposited beside the failure");
  let error: String = verdict
    .expect_err("Expect the sweep to fail for the visual it refused")
    .to_string();

  assert!(error.contains("Failed to fix 1 of 3 ogf visual(s)"), "{error}");

  assert_eq!(result["checked"], 3);
  assert_eq!(result["normalized"], 1);
  assert_eq!(result["unchanged"], 1);
  assert_eq!(result["failed"], 1);
  assert_eq!(result["discardedSize"], fixtures::SPLIT_SIZE);
  assert_eq!(result["files"].as_array().map(Vec::len), Some(1));

  let finding: &Value = &result["findings"][0];

  assert!(
    finding["source"]
      .as_str()
      .is_some_and(|source| source.ends_with("unexplained.ogf")),
    "{finding}"
  );
  assert!(
    finding["message"]
      .as_str()
      .is_some_and(|message| message.contains("remain before source end")),
    "{finding}"
  );

  assert_eq!(fs::read(nested.join("split.OGF"))?, fixtures::well_formed()?);
  assert_eq!(fs::read(root.join("clean.ogf"))?, fixtures::well_formed()?);
  assert_eq!(
    fs::read(root.join("unexplained.ogf"))?,
    fixtures::unexplained_residue()?
  );
  assert_eq!(fs::read(root.join("notes.txt"))?, b"not a visual");

  Ok(())
}

#[test]
fn writes_to_a_destination_and_leaves_the_source_alone() -> XrfResult {
  let root: PathBuf = fixtures::create_root("destination")?;
  let file: PathBuf = root.join("split.ogf");
  let destination: PathBuf = root.join("out").join("fixed.ogf");

  fs::write(&file, fixtures::split_motion_ref()?)?;

  let (verdict, result) = fix(&["--path", &argument(&file), "--dest", &argument(&destination)]);
  let result: Value = result.expect("Expect a result to be deposited");

  verdict.expect("Expect the visual to be fixed");

  assert_eq!(fs::read(&file)?, fixtures::split_motion_ref()?);
  assert_eq!(fs::read(&destination)?, fixtures::well_formed()?);
  assert!(
    result["files"][0]["destination"]
      .as_str()
      .is_some_and(|it| it.ends_with("out/fixed.ogf")),
    "{result}"
  );

  Ok(())
}

/// A well-formed visual sent to a destination is still copied there, so the output exists either way.
#[test]
fn copies_a_well_formed_visual_to_a_destination() -> XrfResult {
  let root: PathBuf = fixtures::create_root("copy")?;
  let file: PathBuf = root.join("clean.ogf");
  let destination: PathBuf = root.join("copy.ogf");

  fs::write(&file, fixtures::well_formed()?)?;

  let (verdict, result) = fix(&["--path", &argument(&file), "--dest", &argument(&destination)]);
  let result: Value = result.expect("Expect a result to be deposited");

  verdict.expect("Expect the copy to succeed");

  assert_eq!(fs::read(&destination)?, fixtures::well_formed()?);
  assert_eq!(result["unchanged"], 1);
  assert_eq!(result["files"], Value::Array(Vec::new()));

  Ok(())
}

#[test]
fn refuses_a_destination_for_a_directory() -> XrfResult {
  let root: PathBuf = fixtures::create_root("directory-destination")?;

  fs::write(root.join("clean.ogf"), fixtures::well_formed()?)?;

  let (verdict, result) = fix(&["--path", &argument(&root), "--dest", &argument(&root.join("out.ogf"))]);
  let error: String = verdict.expect_err("Expect the arguments to be refused").to_string();

  assert!(error.contains("Destination applies to a single ogf file"), "{error}");
  assert!(result.is_none(), "Expect nothing to be reported for a refused run");
  assert_eq!(names_in(&root)?, vec![String::from("clean.ogf")]);

  Ok(())
}

#[test]
fn refuses_a_directory_holding_no_visuals() -> XrfResult {
  let root: PathBuf = fixtures::create_root("empty")?;

  fs::write(root.join("notes.txt"), b"not a visual")?;

  let error: String = fix(&["--path", &argument(&root)])
    .0
    .expect_err("Expect an empty sweep to be refused")
    .to_string();

  assert!(error.contains("No ogf visuals found"), "{error}");

  Ok(())
}

/// A write that fails part way leaves the source as it was, not truncated, and no staging file beside it.
#[test]
fn keeps_the_source_when_the_write_fails() -> XrfResult {
  let root: PathBuf = fixtures::create_root("failed-write")?;
  let file: PathBuf = root.join("split.ogf");

  fs::write(&file, fixtures::split_motion_ref()?)?;
  fail_next_staged_write();

  let (verdict, result) = fix(&["--path", &argument(&file)]);
  let result: Value = result.expect("Expect a result to be deposited beside the failure");

  verdict.expect_err("Expect the run to fail for the visual it could not write");

  assert_eq!(fs::read(&file)?, fixtures::split_motion_ref()?);
  assert_eq!(result["failed"], 1);
  assert!(
    result["findings"][0]["message"]
      .as_str()
      .is_some_and(|message| message.contains("File was not written")),
    "{result}"
  );
  assert_eq!(names_in(&root)?, vec![String::from("split.ogf")]);

  Ok(())
}
