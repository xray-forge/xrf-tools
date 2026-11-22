//! The serialized report, which the CLI's JSON reports, their e2e snapshots and the app's TypeScript mirrors read.

use std::time::Duration;

use serde_json::{Value, json};

use crate::check_id::CheckId;
use crate::check_report::CheckReport;
use crate::finding::Finding;
use crate::report::Report;
use crate::rule_id::RuleId;
use crate::severity::Severity;
use crate::status::Status;

#[test]
fn serializes_in_the_shape_its_readers_expect() {
  let report: Report = Report::new(vec![
    CheckReport::new(
      CheckId::of_static("textures"),
      Status::Failed,
      Some(Duration::from_millis(1500)),
      vec![
        Finding::warning(
          RuleId::of_static("textures.missing"),
          Some(String::from("a.dds")),
          "Missing",
        ),
        Finding::failure(RuleId::of_static("textures.read"), None, "Broken"),
      ],
    ),
    CheckReport::new(CheckId::of_static("scripts"), Status::Skipped, None, Vec::new()),
  ]);

  // Keys come out alphabetical, as the fields are declared, so a report diffs the same however it was built.
  assert_eq!(
    serde_json::to_string(&report).unwrap(),
    json!({
      "checks": [
        {
          "duration": 1500,
          "findings": [
            { "message": "Broken", "ruleId": "textures.read", "severity": "failure", "subject": null },
            { "message": "Missing", "ruleId": "textures.missing", "severity": "warning", "subject": "a.dds" },
          ],
          "id": "textures",
          "status": "failed",
        },
        { "duration": null, "findings": [], "id": "scripts", "status": "skipped" },
      ],
      "status": "failed",
    })
    .to_string()
  );
}

#[test]
fn names_each_status_and_severity_as_it_serializes() {
  for status in [
    Status::Error,
    Status::Failed,
    Status::Incomplete,
    Status::Passed,
    Status::Skipped,
  ] {
    assert_eq!(serde_json::to_value(status).unwrap(), Value::String(status.to_string()));
  }

  for severity in [Severity::Failure, Severity::Warning] {
    assert_eq!(
      serde_json::to_value(severity).unwrap(),
      Value::String(severity.to_string())
    );
  }
}

fn warning() -> Finding {
  Finding::warning(RuleId::of_static("dialog.text-missing"), None, "Shown as its key")
}

fn failure() -> Finding {
  Finding::failure(RuleId::of_static("dialog.offer-missing"), None, "Asserts")
}

#[test]
fn judges_a_check_by_its_failures_alone() {
  let judge = |findings: Vec<Finding>| CheckReport::judge(CheckId::of_static("dialogs"), None, findings).status();

  assert_eq!(judge(Vec::new()), Status::Passed);
  assert_eq!(judge(vec![warning()]), Status::Passed);
  assert_eq!(judge(vec![warning(), failure()]), Status::Failed);
}

/// Strict promotes a pass with warnings and nothing else: a skipped, stopped or broken check says the same either way.
#[test]
fn fails_a_check_that_passed_with_warnings_only_when_strict() {
  let check =
    |status: Status, findings: Vec<Finding>| CheckReport::new(CheckId::of_static("dialogs"), status, None, findings);

  let lenient: Report = Report::new(vec![
    check(Status::Passed, vec![warning()]),
    check(Status::Passed, Vec::new()),
    check(Status::Skipped, vec![warning()]),
  ]);

  assert_eq!(lenient.status(), Status::Passed);
  assert_eq!(lenient.count_findings(), 2);

  let strict: Report = lenient.into_strict();

  assert_eq!(strict.status(), Status::Failed);
  assert_eq!(
    strict.checks().iter().map(CheckReport::status).collect::<Vec<Status>>(),
    [Status::Failed, Status::Passed, Status::Skipped]
  );
  // The finding keeps what it is; only the verdict it leads to changes.
  assert_eq!(strict.checks()[0].findings()[0].severity(), Severity::Warning);
}
