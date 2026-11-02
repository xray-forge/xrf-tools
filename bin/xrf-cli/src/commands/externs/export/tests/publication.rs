//! What a writing `externs export` leaves behind when publication succeeds and when it fails.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;
use xrf_utils::StagingFaults;

use crate::commands::externs::export::command::ExportCommand;
use crate::core::command::command_testing::run_command_with_result;
use crate::core::command::{CommandError, CommandResult};

const PREVIOUS: &str = "{\n  \"exports\": { \"sentinel\": true }\n}\n";

/// A declarations root beside an artifact a previous run already published.
fn create_root(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = GeneratedResources::create_dir(&format!("externs-export-publication/{name}"))?;

  fs::create_dir_all(root.join("declarations"))?;
  fs::write(
    root.join("declarations/externs.ts"),
    "export {}; extern(\"callbacks.run\", (): void => {});",
  )?;
  fs::write(root.join("externs.json"), PREVIOUS)?;

  Ok(root)
}

fn export(root: &Path) -> CommandResult {
  run_command_with_result(
    &ExportCommand,
    &[
      String::from("export"),
      String::from("-p"),
      root.join("declarations").display().to_string(),
      String::from("-d"),
      root.join("externs.json").display().to_string(),
      String::from("--json"),
    ],
  )
  .0
}

#[test]
fn replaces_an_artifact_a_previous_run_published() -> XrfResult {
  let root: PathBuf = create_root("replaces")?;

  export(&root).expect("the artifact to be published");

  let written: String = fs::read_to_string(root.join("externs.json"))?;

  assert!(!written.contains("sentinel"), "{written}");
  assert!(written.contains("callbacks.run"), "{written}");

  Ok(())
}

#[test]
fn keeps_the_previous_artifact_when_publication_fails() -> XrfResult {
  let root: PathBuf = create_root("failed")?;

  StagingFaults::fail_next_write();

  // A failed publication is the command's own failure, not a verdict on the declarations.
  match export(&root) {
    Err(error @ CommandError::Execution(_)) => {
      assert_eq!(error.exit_code(), 1);
      assert!(error.message().contains("externs.json"), "{error}");
    }
    other => panic!("expected an execution failure, got {other:?}"),
  }

  assert_eq!(fs::read_to_string(root.join("externs.json"))?, PREVIOUS);

  // Only the artifact and the declarations remain; nothing staged was left in the way.
  let mut names: Vec<String> = fs::read_dir(&root)?
    .filter_map(Result::ok)
    .map(|entry| entry.file_name().to_string_lossy().to_string())
    .collect();

  names.sort();

  assert_eq!(names, vec![String::from("declarations"), String::from("externs.json")]);

  Ok(())
}
