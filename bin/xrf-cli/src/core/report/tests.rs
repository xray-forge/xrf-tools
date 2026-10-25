use std::fs;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::time::Duration;

use serde_json::{Value, json};
use xrf_build_info::{BuildInfo, BuildKind};
use xrf_error::XrfError;
use xrf_job::{ExecutionPlan, ExecutionRequest};
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_utils::staging_faults::fail_next_staged_write;

use crate::core::command::{CommandError, CommandResult};
use crate::core::report::{CommandEnvelope, ReportDestination};

/// The plan a test envelope carries, where the width is not what the test is about.
fn plan() -> ExecutionPlan {
  ExecutionRequest::Workers(NonZeroUsize::new(1).expect("one is not zero")).resolve()
}

/// A build that recorded nothing beyond its own version, so an expectation names only what it set.
fn build() -> BuildInfo {
  BuildInfo {
    version: "1.2.3",
    kind: BuildKind::Local,
    commit: None,
    reference: None,
    is_dirty: false,
    built_at: None,
    target: None,
    rustc: None,
    profile: None,
    optimization: None,
    run_id: None,
  }
}

/// The envelope as a document, less the two fields that describe the run rather than its outcome.
///
/// Removing them rather than restating them in every expectation keeps each test about the field it names, and each
/// removal doubles as the assertion that every envelope carries one.
fn envelope(outcome: CommandResult, result: Option<Value>) -> Value {
  let envelope: CommandEnvelope = CommandEnvelope::new(
    &build(),
    vec![String::from("archive"), String::from("info")],
    &outcome,
    Duration::from_millis(1200),
    &plan(),
    result,
  );

  let mut document: Value = serde_json::to_value(envelope).expect("envelope to serialize");

  let fields = document
    .as_object_mut()
    .expect("the envelope to serialize as an object");

  fields
    .remove("build")
    .expect("every envelope to carry the build that produced it");
  fields
    .remove("execution")
    .expect("every envelope to carry the width the run was bounded to");

  document
}

#[test]
fn leaves_stdout_to_the_document_only_in_pipe_mode() {
  assert!(ReportDestination::None.is_stdout_available());
  assert!(ReportDestination::File(PathBuf::from("out.json")).is_stdout_available());
  assert!(!ReportDestination::Stdout.is_stdout_available());
}

#[test]
fn asks_for_a_structured_result_only_when_one_will_be_read() {
  assert!(!ReportDestination::None.is_requested());
  assert!(ReportDestination::Stdout.is_requested());
  assert!(ReportDestination::File(PathBuf::from("out.json")).is_requested());
}

#[test]
fn records_a_successful_run() {
  assert_eq!(
    envelope(Ok(()), Some(json!({ "volumes": 2 }))),
    json!({
      "command": ["archive", "info"],
      "duration": 1200,
      "exitCode": 0,
      "result": { "volumes": 2 },
    })
  );
}

/// A check that failed finished the run: its result says why, and an `error` beside it would only restate the exit code.
#[test]
fn records_a_check_failure_by_its_result_and_exit_code() {
  assert_eq!(
    envelope(
      Err(CommandError::new_check_failed(3)),
      Some(json!({ "status": "failed" }))
    ),
    json!({
      "command": ["archive", "info"],
      "duration": 1200,
      "exitCode": 3,
      "result": { "status": "failed" },
    })
  );
}

#[test]
fn records_an_execution_failure_without_a_result() {
  assert_eq!(
    envelope(Err(CommandError::from(XrfError::new_generic_error("boom"))), None),
    json!({
      "command": ["archive", "info"],
      "duration": 1200,
      "error": "Generic error: boom",
      "exitCode": 1,
    })
  );
}

/// The same argument as the build block: a report outlives its run, and comparing two of them means knowing how wide
/// each one ran. Nothing else in the envelope, and nothing in the exit contract, can answer that afterwards.
#[test]
fn records_the_width_the_run_was_bounded_to() {
  let envelope: CommandEnvelope = CommandEnvelope::new(
    &build(),
    vec![String::from("gamedata"), String::from("verify")],
    &Ok(()),
    Duration::from_millis(1200),
    &ExecutionRequest::Workers(NonZeroUsize::new(4).expect("four is not zero")).resolve(),
    None,
  );
  let document: Value = serde_json::to_value(envelope).expect("envelope to serialize");

  // Carried by every run, including the commands that declare no `--jobs` of their own, because every command runs
  // inside a pool whether or not it offers a say in how wide that pool is.
  assert_eq!(document["execution"], json!({ "workers": 4 }));
}

/// A report outlives its run, and nothing else in the envelope says which binary wrote it.
#[test]
fn records_the_binary_that_produced_the_run() {
  let envelope: CommandEnvelope = CommandEnvelope::new(
    &build(),
    vec![String::from("archive"), String::from("info")],
    &Ok(()),
    Duration::from_millis(1200),
    &plan(),
    None,
  );
  let document: Value = serde_json::to_value(envelope).expect("envelope to serialize");

  // Only what tells two runs' binaries apart; a build recording no commit, profile or CI run omits them.
  assert_eq!(
    document["build"],
    json!({
      "version": "1.2.3",
      "kind": "local",
      "isDirty": false,
    })
  );
}

#[test]
fn publishes_nothing_when_no_report_was_asked_for() {
  let envelope: CommandEnvelope = CommandEnvelope::new(&build(), Vec::new(), &Ok(()), Duration::ZERO, &plan(), None);

  assert!(envelope.publish(&ReportDestination::None).is_ok());
}

/// A destination of this test's own, since these cases are about what a file holds afterwards.
fn report_path(name: &str) -> PathBuf {
  let directory: PathBuf = build_absolute_generated_test_resource_path(&format!("core/reporting/{name}"));

  fs::create_dir_all(&directory).expect("the report directory to be created");

  directory.join("report.json")
}

#[test]
fn replaces_a_report_a_previous_run_wrote() {
  let path: PathBuf = report_path("replaces");
  let envelope: CommandEnvelope = CommandEnvelope::new(&build(), Vec::new(), &Ok(()), Duration::ZERO, &plan(), None);

  fs::write(&path, b"{\"sentinel\":true}").expect("the previous report to be seeded");

  envelope
    .publish(&ReportDestination::File(path.clone()))
    .expect("the report to be published");

  let written: String = fs::read_to_string(&path).expect("the report to be readable");

  assert!(!written.contains("sentinel"), "{written}");
  assert!(written.contains("\"exitCode\": 0"), "{written}");
}

/// The contract the staging exists for: a report that could not be delivered destroys nothing.
#[test]
fn keeps_the_previous_report_when_publication_fails() {
  let path: PathBuf = report_path("preserves");
  let envelope: CommandEnvelope = CommandEnvelope::new(&build(), Vec::new(), &Ok(()), Duration::ZERO, &plan(), None);

  fs::write(&path, b"{\"sentinel\":true}").expect("the previous report to be seeded");
  fail_next_staged_write();

  let error: CommandError = envelope
    .publish(&ReportDestination::File(path.clone()))
    .expect_err("an undeliverable report to fail the run");

  assert_eq!(error.exit_code(), 1);
  assert_eq!(
    fs::read(&path).expect("the report to be readable"),
    b"{\"sentinel\":true}"
  );
}

#[test]
fn fails_the_run_when_a_requested_report_cannot_be_written() {
  let envelope: CommandEnvelope = CommandEnvelope::new(&build(), Vec::new(), &Ok(()), Duration::ZERO, &plan(), None);
  let destination: ReportDestination = ReportDestination::File(PathBuf::from("missing-directory/report.json"));

  let error: CommandError = envelope
    .publish(&destination)
    .expect_err("an unwritable report to fail the run");

  assert_eq!(error.exit_code(), 1);
  assert!(error.message().contains("missing-directory"), "{}", error.message());
}
