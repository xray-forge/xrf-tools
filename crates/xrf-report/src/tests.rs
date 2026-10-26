//! The serialized report, which the CLI's JSON reports, their e2e snapshots and the app's TypeScript mirrors read.

use std::time::Duration;

use serde_json::{Value, json};

use crate::{CheckId, CheckReport, Finding, Report, RuleId, Status};

#[test]
fn serializes_in_the_shape_its_readers_expect() {
  let report: Report = Report::new(vec![
    CheckReport::new(
      CheckId::of_static("textures"),
      Status::Failed,
      Some(Duration::from_millis(1500)),
      vec![
        Finding::new(
          RuleId::of_static("textures.missing"),
          Some(String::from("a.dds")),
          "Missing",
        ),
        Finding::new(RuleId::of_static("textures.read"), None, "Broken"),
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
            { "message": "Broken", "ruleId": "textures.read", "subject": null },
            { "message": "Missing", "ruleId": "textures.missing", "subject": "a.dds" },
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
fn names_each_status_as_it_serializes() {
  for status in [
    Status::Error,
    Status::Failed,
    Status::Incomplete,
    Status::Passed,
    Status::Skipped,
  ] {
    assert_eq!(serde_json::to_value(status).unwrap(), Value::String(status.to_string()));
  }
}
