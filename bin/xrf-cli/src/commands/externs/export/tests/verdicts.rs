//! Which `externs export` failures are a verdict on content and which are the command's own.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use xrf_error::XrfResult;
use xrf_test_utils::utils::create_generated_test_resource_dir;

use crate::commands::externs::export::command::ExportCommand;
use crate::core::command::command_testing::run_command_with_result;
use crate::core::command::{CommandError, CommandResult};

/// A declarations root holding one source, beside the artifact a `--check` run compares against.
fn create_root(name: &str, declarations: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = create_generated_test_resource_dir(&format!("externs-export/{name}"))?;

  fs::create_dir_all(root.join("declarations"))?;
  fs::write(root.join("declarations/externs.ts"), declarations)?;
  fs::write(root.join("externs.json"), "{\n  \"exports\": {}\n}\n")?;

  Ok(root)
}

/// Run a check over a root, as the process would, and keep what it deposited alongside its verdict.
fn check(root: &Path) -> (CommandResult, Option<Value>) {
  run_command_with_result(
    &ExportCommand,
    &[
      String::from("export"),
      String::from("-p"),
      root.join("declarations").display().to_string(),
      String::from("-d"),
      root.join("externs.json").display().to_string(),
      String::from("--check"),
      String::from("--json"),
    ],
  )
}

/// The single finding a failed check reported, with the report shape that carries it.
fn finding(verdict: CommandResult, result: Option<Value>) -> String {
  match verdict {
    Err(CommandError::CheckFailed { findings }) => assert_eq!(findings, 1),
    other => panic!("expected a check failure, got {other:?}"),
  }

  let result: Value = result.expect("a report to be deposited");
  let finding: &Value = &result["checks"][0]["findings"][0];

  assert_eq!(result["status"], "failed");
  assert_eq!(result["externs"], 0);
  assert_eq!(finding["ruleId"], "externs.declarations");

  finding["message"].as_str().expect("the finding to be reported").into()
}

#[test]
fn judges_a_duplicate_declaration_instead_of_failing_the_run() -> XrfResult {
  let root: PathBuf = create_root(
    "duplicate",
    "export {}; extern(\"callbacks.run\", (): void => {}); extern(\"callbacks.run\", (): void => {});",
  )?;

  // A name declared twice is the declarations' own defect, which the failure contract answers with 3.
  let (verdict, result) = check(&root);
  let error: String = finding(verdict, result);

  assert!(error.contains("Duplicate extern 'callbacks.run'"), "{error}");

  Ok(())
}

#[test]
fn judges_malformed_typescript_and_keeps_the_position_it_reported() -> XrfResult {
  let root: PathBuf = create_root("malformed", "export {}; extern(\"callbacks.run\", (): void => );")?;

  let (verdict, result) = check(&root);
  let error: String = finding(verdict, result);

  assert!(
    error.starts_with("Parsing error: Failed to parse TypeScript "),
    "{error}"
  );
  assert!(error.contains("externs.ts:1:"), "{error}");

  Ok(())
}

#[test]
fn refuses_a_missing_declarations_root_as_an_execution_failure() -> XrfResult {
  let root: PathBuf = create_root("missing-root", "export {};")?;

  fs::remove_dir_all(root.join("declarations"))?;

  // Nothing was judged, so this is the command failing rather than a verdict on content.
  match check(&root).0 {
    Err(error @ CommandError::Execution(_)) => {
      assert_eq!(error.exit_code(), 1);
      assert!(error.message().contains("is not a directory"), "{error}");
    }
    other => panic!("expected an execution failure, got {other:?}"),
  }

  Ok(())
}

#[test]
fn keeps_invalid_declarations_an_execution_failure_when_writing() -> XrfResult {
  let root: PathBuf = create_root(
    "writing",
    "export {}; extern(\"callbacks.run\", (): void => {}); extern(\"callbacks.run\", (): void => {});",
  )?;

  // Writing judges nothing, so the same declarations have no verdict to fail.
  let (verdict, _) = run_command_with_result(
    &ExportCommand,
    &[
      String::from("export"),
      String::from("-p"),
      root.join("declarations").display().to_string(),
      String::from("-d"),
      root.join("written.json").display().to_string(),
      String::from("--json"),
    ],
  );

  match verdict {
    Err(error @ CommandError::Execution(_)) => assert_eq!(error.exit_code(), 1),
    other => panic!("expected an execution failure, got {other:?}"),
  }

  assert!(!root.join("written.json").exists());

  Ok(())
}
