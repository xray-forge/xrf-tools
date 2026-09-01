//! Exercises the `translation format` command end to end.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use xrf_error::XrfResult;

use crate::commands::translation::format::command::FormatCommand;
use crate::core::command_testing::{run_command, run_command_with_result};
use crate::core::generic_command::CommandResult;

const UNSORTED: &str = "{\n  \"st_b\": {\n    \"eng\": \"B\"\n  },\n  \"st_a\": {\n    \"eng\": \"A\"\n  }\n}\n";
const CANONICAL: &str = "{\n  \"st_a\": {\n    \"eng\": \"A\"\n  },\n  \"st_b\": {\n    \"eng\": \"B\"\n  }\n}\n";

fn create_root(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = std::env::temp_dir().join(format!("xrf-cli-format-translation-{name}-{}", std::process::id()));

  if root.exists() {
    fs::remove_dir_all(&root)?;
  }

  fs::create_dir_all(&root)?;

  Ok(root)
}

fn arguments(path: &Path, rest: &[&str]) -> Vec<String> {
  let mut arguments: Vec<String> = vec![
    String::from("format"),
    String::from("--path"),
    path.display().to_string(),
    String::from("--silent"),
  ];

  arguments.extend(rest.iter().map(|value| String::from(*value)));
  arguments
}

#[test]
fn normalizes_a_source_and_reports_it() -> CommandResult {
  let root: PathBuf = create_root("normalize")?;
  let file: PathBuf = root.join("st_a.json");

  fs::write(&file, UNSORTED)?;

  let (verdict, result) = run_command_with_result(&FormatCommand, &arguments(&root, &["--json"]));

  verdict?;

  assert_eq!(fs::read_to_string(&file)?, CANONICAL);

  let result: Value = result.expect("Expected a deposited result");

  assert_eq!(result["invalidFiles"], 1);
  assert_eq!(result["validFiles"], 0);
  assert_eq!(result["totalFiles"], 1);
  assert_eq!(result["toFormat"].as_array().expect("Expected a list").len(), 1);
  assert_eq!(result["outcome"], "completed");

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn a_check_fails_without_writing() -> CommandResult {
  let root: PathBuf = create_root("check")?;
  let file: PathBuf = root.join("st_a.json");

  fs::write(&file, UNSORTED)?;

  let (verdict, result) = run_command_with_result(&FormatCommand, &arguments(&root, &["--check", "--json"]));

  // Judged content, so exit 3 rather than an execution failure.
  assert_eq!(
    verdict
      .expect_err("Expected an unformatted source to fail the check")
      .exit_code(),
    3
  );
  assert_eq!(fs::read_to_string(&file)?, UNSORTED);
  assert_eq!(
    result.expect("Expected findings to be deposited before the verdict")["invalidFiles"],
    1
  );

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn a_check_over_a_canonical_tree_succeeds() -> CommandResult {
  let root: PathBuf = create_root("clean")?;

  fs::write(root.join("st_a.json"), CANONICAL)?;

  run_command(&FormatCommand, &arguments(&root, &["--check"]))?;

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn formatting_is_idempotent() -> CommandResult {
  let root: PathBuf = create_root("idempotent")?;
  let file: PathBuf = root.join("st_a.json");

  fs::write(&file, UNSORTED)?;

  run_command(&FormatCommand, &arguments(&root, &[]))?;

  let once: String = fs::read_to_string(&file)?;

  run_command(&FormatCommand, &arguments(&root, &[]))?;

  assert_eq!(fs::read_to_string(&file)?, once);
  // And the second run has nothing left to do, which is what makes it safe as a gate.
  run_command(&FormatCommand, &arguments(&root, &["--check"]))?;

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn an_asserted_line_ending_is_written_and_judged() -> CommandResult {
  let root: PathBuf = create_root("endings")?;
  let file: PathBuf = root.join("st_a.json");

  fs::write(&file, CANONICAL)?;

  // Without the flag the file is already formatted, endings unjudged.
  run_command(&FormatCommand, &arguments(&root, &["--check"]))?;

  // With it, the same file is not.
  assert_eq!(
    run_command(
      &FormatCommand,
      &arguments(&root, &["--check", "--line-endings", "crlf"])
    )
    .expect_err("Expected an asserted ending to arm the check")
    .exit_code(),
    3
  );

  run_command(&FormatCommand, &arguments(&root, &["--line-endings", "crlf"]))?;

  assert!(fs::read_to_string(&file)?.contains("\r\n"));

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn an_unusable_line_ending_is_refused_before_anything_is_written() -> CommandResult {
  let root: PathBuf = create_root("bad-endings")?;
  let file: PathBuf = root.join("st_a.json");

  fs::write(&file, UNSORTED)?;

  // Clap rejects the value against its list, so nothing reaches the formatter. The exit code is not asserted here:
  // `CommandError` carries only execution and check failures, and the usage exit is `application.rs`'s to produce, so
  // this harness renders a clap refusal as exit 1 where the process renders it as exit 2.
  run_command(&FormatCommand, &arguments(&root, &["--line-endings", "cr"]))
    .expect_err("Expected an unsupported line ending to be refused");

  assert_eq!(fs::read_to_string(&file)?, UNSORTED);

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn a_malformed_source_is_an_execution_failure() -> CommandResult {
  let root: PathBuf = create_root("malformed")?;

  fs::write(root.join("st_a.json"), "{ not json")?;

  // Exit 1, not a finding: a formatter has nothing to write except what it read.
  assert_eq!(
    run_command(&FormatCommand, &arguments(&root, &["--check"]))
      .expect_err("Expected a malformed source to stop the run")
      .exit_code(),
    1
  );

  fs::remove_dir_all(root)?;

  Ok(())
}

#[test]
fn a_tree_holding_no_sources_is_an_execution_failure() -> CommandResult {
  let root: PathBuf = create_root("empty")?;

  fs::write(root.join("notes.txt"), "ignored")?;

  // Unlike `ltx format`, which formats an empty set successfully. A renamed directory would otherwise make this gate
  // pass over nothing at all.
  assert_eq!(
    run_command(&FormatCommand, &arguments(&root, &["--check"]))
      .expect_err("Expected an empty selection to be refused")
      .exit_code(),
    1
  );

  fs::remove_dir_all(root)?;

  Ok(())
}
