use std::path::Path;
use std::time::Duration;

use serde::Serialize;
use xrf_engine_target::XrayEngineResolution;
use xrf_gamedata::GamedataVerificationReport;
use xrf_report::Status;
use xrf_vfs::{XrayCacheStats, XrayReadTraceSummary, XraySkippedMount};

use super::gamedata_verify_check_report::GamedataVerifyCheckReport;
use crate::core::reports::SkippedMountReport;

/// What a `gamedata verify` run reports to a machine.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamedataVerifyReport {
  /// What retention held and how it served the sweep, which no timing can show.
  cache: XrayCacheStats,
  checks: Vec<GamedataVerifyCheckReport>,
  #[serde(with = "xrf_utils::duration_ms")]
  duration: Duration,
  /// The engine the configs were read as, and what decided it.
  engine: XrayEngineResolution,
  /// What the sweep physically read, present only when `--trace-reads` asked for the account.
  #[serde(skip_serializing_if = "Option::is_none")]
  reads: Option<XrayReadTraceSummary>,
  /// Declared sources that never opened, which is what the `incomplete` status is about.
  skipped_mounts: Vec<SkippedMountReport>,
  status: Status,
}

impl GamedataVerifyReport {
  /// `result` as verified under `root`, with what the project's mount and reads said about the run.
  pub fn new(
    root: &Path,
    engine: &XrayEngineResolution,
    result: &GamedataVerificationReport,
    skipped: &[XraySkippedMount],
    cache: XrayCacheStats,
    reads: Option<XrayReadTraceSummary>,
  ) -> Self {
    Self {
      cache,
      checks: result
        .get_checks()
        .iter()
        .map(|check| GamedataVerifyCheckReport::new(root, check))
        .collect(),
      duration: result.get_duration(),
      engine: engine.clone(),
      reads,
      skipped_mounts: SkippedMountReport::list(skipped),
      status: result.get_status(),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::path::{Path, PathBuf};
  use std::time::Duration;

  use xrf_engine_target::XrayEngineResolution;
  use xrf_gamedata::{
    Finding, GamedataCheckResult, GamedataVerificationReport, GamedataVerificationStatus, GamedataVerificationType,
  };
  use xrf_report::RuleId;
  use xrf_vfs::{XrayCacheStats, XrayReadTraceHotPath, XrayReadTraceSummary, XraySkippedMount};

  use super::GamedataVerifyReport;

  struct TestCheckResult {
    duration: Duration,
    findings: Vec<Finding>,
  }

  impl GamedataCheckResult for TestCheckResult {
    fn get_duration(&self) -> Option<Duration> {
      Some(self.duration)
    }

    fn get_status(&self) -> GamedataVerificationStatus {
      GamedataVerificationStatus::Failed
    }

    fn get_summary(&self) -> String {
      String::from("2/2 textures are invalid")
    }

    fn get_findings(&self) -> &[Finding] {
      &self.findings
    }
  }

  /// Nothing is read from the root: it only decides how the findings name their assets.
  fn root() -> PathBuf {
    PathBuf::from("gamedata-root")
  }

  fn to_json(root: &Path, report: &GamedataVerificationReport, skipped: &[XraySkippedMount]) -> serde_json::Value {
    serde_json::to_value(GamedataVerifyReport::new(
      root,
      &XrayEngineResolution::undetected(),
      report,
      skipped,
      XrayCacheStats::default(),
      None,
    ))
    .unwrap()
  }

  #[test]
  fn reports_checks_in_the_shared_shape_with_root_relative_subjects() {
    let root: PathBuf = root();
    let mut report: GamedataVerificationReport = GamedataVerificationReport::with_duration(Duration::from_millis(42));

    report.add_check(
      GamedataVerificationType::Textures,
      Ok(TestCheckResult {
        duration: Duration::from_millis(7),
        findings: vec![
          Finding::new(
            RuleId::of_static("textures.dds"),
            Some(root.join("textures").join("z.dds").display().to_string()),
            "Second finding",
          ),
          Finding::new(
            RuleId::of_static("textures.dds"),
            Some(root.join("textures").join("a.dds").display().to_string()),
            "First finding",
          ),
        ],
      }),
    );

    let json: serde_json::Value = to_json(&root, &report, &[]);

    assert_eq!(json["status"], "failed");
    assert_eq!(json["duration"], 42);
    assert_eq!(
      json["engine"],
      serde_json::json!({ "engine": "vanilla", "evidence": "noSigns", "subject": null })
    );
    assert_eq!(
      json["checks"][0],
      serde_json::json!({
        "duration": 7,
        "findings": [
          { "message": "First finding", "ruleId": "textures.dds", "subject": "textures/a.dds" },
          { "message": "Second finding", "ruleId": "textures.dds", "subject": "textures/z.dds" },
        ],
        "id": "textures",
        "status": "failed",
        "summary": "2/2 textures are invalid",
      })
    );
  }

  /// Retention is invisible to every timing the report already carries, so it is reported rather than inferred.
  #[test]
  fn reports_how_retention_served_the_sweep() {
    let report: GamedataVerificationReport = GamedataVerificationReport::with_duration(Duration::from_millis(42));
    let cache: XrayCacheStats = XrayCacheStats {
      entries: 3,
      bytes: 4096,
      hits: 12,
      misses: 5,
      refused: 1,
    };

    let json: serde_json::Value = serde_json::to_value(GamedataVerifyReport::new(
      &root(),
      &XrayEngineResolution::undetected(),
      &report,
      &[],
      cache,
      None,
    ))
    .unwrap();

    assert_eq!(
      json["cache"],
      serde_json::json!({
        "entries": 3,
        "bytes": 4096,
        "hits": 12,
        "misses": 5,
        "refused": 1,
      })
    );

    // The account is opt-in, and a zeroed one would read as a run that read nothing.
    assert!(json.get("reads").is_none());
  }

  /// `--trace-reads` is the only thing that can say a run read the same bytes twice.
  #[test]
  fn reports_the_read_account_when_one_was_asked_for() {
    let report: GamedataVerificationReport = GamedataVerificationReport::with_duration(Duration::from_millis(42));
    let reads: XrayReadTraceSummary = XrayReadTraceSummary {
      paths: 2,
      reads: 4,
      bytes: 3040,
      unique_bytes: 1040,
      hottest: vec![XrayReadTraceHotPath {
        path: String::from("anims\\shared.omf"),
        reads: 3,
        bytes: 3000,
      }],
    };

    let json: serde_json::Value = serde_json::to_value(GamedataVerifyReport::new(
      &root(),
      &XrayEngineResolution::undetected(),
      &report,
      &[],
      XrayCacheStats::default(),
      Some(reads),
    ))
    .unwrap();

    assert_eq!(json["reads"]["paths"], 2);
    assert_eq!(json["reads"]["reads"], 4);
    assert_eq!(json["reads"]["bytes"], 3040);
    assert_eq!(json["reads"]["uniqueBytes"], 1040);
    assert_eq!(json["reads"]["hottest"][0]["path"], "anims\\shared.omf");
    assert_eq!(json["reads"]["hottest"][0]["reads"], 3);
  }

  /// What a script reads to learn the run did not cover everything, without parsing a finding message.
  #[test]
  fn reports_every_declared_source_that_never_opened() {
    let root: PathBuf = root();
    let report: GamedataVerificationReport = GamedataVerificationReport::with_duration(Duration::from_millis(42));
    let skipped: Vec<XraySkippedMount> = vec![XraySkippedMount {
      origin: String::from("$arch_dir$"),
      path: root.join("db"),
      reason: String::from("Failed to read archive volumes"),
    }];

    let json: serde_json::Value = to_json(&root, &report, &skipped);

    // The whole address, not a root-relative one: a source that never opened may sit outside the verified root, and a
    // reader has to be able to find the path the plan named.
    assert_eq!(
      json["skippedMounts"][0],
      serde_json::json!({
        "origin": "$arch_dir$",
        "path": xrf_utils::to_portable_path_string(root.join("db")),
        "reason": "Failed to read archive volumes",
      })
    );
  }

  /// Empty rather than absent, or a consumer cannot tell "nothing skipped" from "this build does not report it".
  #[test]
  fn reports_full_coverage_as_an_empty_list() {
    let report: GamedataVerificationReport = GamedataVerificationReport::with_duration(Duration::from_millis(42));

    let json: serde_json::Value = to_json(&root(), &report, &[]);

    assert_eq!(json["skippedMounts"], serde_json::json!([]));
  }
}
