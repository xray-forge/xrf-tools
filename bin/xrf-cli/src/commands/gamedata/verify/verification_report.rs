use std::path::Path;
use std::time::Duration;

use serde::Serialize;
use xrf_gamedata::{GamedataVerificationCheckReport, GamedataVerificationResult};
use xrf_report::{CheckReport, Finding};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamedataVerificationReportOutput {
  checks: Vec<GamedataVerificationCheckReportOutput>,
  #[serde(with = "xrf_utils::duration_ms")]
  duration: Duration,
  status: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GamedataVerificationCheckReportOutput {
  #[serde(with = "xrf_utils::optional_duration_ms")]
  duration: Option<Duration>,
  findings: Vec<GamedataVerificationFindingOutput>,
  status: String,
  summary: String,
  verification_type: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GamedataVerificationFindingOutput {
  asset_path: Option<String>,
  message: String,
  rule_id: String,
}

/// Builds what a `gamedata verify` run reports to a machine.
///
/// Findings name assets relative to the verified root, which is the only thing here a raw
/// verification result cannot say on its own.
pub struct GamedataVerificationReportPayload<'a> {
  root: &'a Path,
  report: &'a GamedataVerificationResult,
}

impl<'a> GamedataVerificationReportPayload<'a> {
  pub fn new(root: &'a Path, report: &'a GamedataVerificationResult) -> Self {
    Self { root, report }
  }

  pub fn build(&self) -> GamedataVerificationReportOutput {
    let checks: Vec<GamedataVerificationCheckReportOutput> = self
      .report
      .get_checks()
      .iter()
      .map(|gamedata_check| self.check_report_output(gamedata_check, gamedata_check.get_report()))
      .collect();

    GamedataVerificationReportOutput {
      checks,
      duration: self.report.get_duration(),
      status: self.report.get_status().to_string(),
    }
  }

  fn check_report_output(
    &self,
    gamedata_report: &GamedataVerificationCheckReport,
    report: &CheckReport,
  ) -> GamedataVerificationCheckReportOutput {
    let findings: Vec<GamedataVerificationFindingOutput> = report
      .findings()
      .iter()
      .map(|finding| self.finding_output(finding))
      .collect();

    GamedataVerificationCheckReportOutput {
      duration: report.duration(),
      findings,
      status: report.status().to_string(),
      summary: gamedata_report.get_summary().to_string(),
      verification_type: report.id().to_string(),
    }
  }

  fn finding_output(&self, finding: &Finding) -> GamedataVerificationFindingOutput {
    GamedataVerificationFindingOutput {
      asset_path: finding.subject().map(|subject| {
        let asset_path: &Path = Path::new(subject);

        asset_path
          .strip_prefix(self.root)
          .unwrap_or(asset_path)
          .to_string_lossy()
          .replace('\\', "/")
      }),
      message: finding.message().to_string(),
      rule_id: finding.rule_id().to_string(),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;
  use std::sync::atomic::{AtomicU64, Ordering};
  use std::time::Duration;

  use xrf_gamedata::{
    Finding, GamedataCheckResult, GamedataVerificationReport, GamedataVerificationStatus, GamedataVerificationType,
  };
  use xrf_report::RuleId;

  use super::GamedataVerificationReportPayload;

  static NEXT_TEST_DIRECTORY_ID: AtomicU64 = AtomicU64::new(0);

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

    fn get_failure_message(&self) -> String {
      String::from("2/2 textures are invalid")
    }

    fn get_findings(&self) -> &[Finding] {
      &self.findings
    }
  }

  fn temporary_gamedata_root() -> PathBuf {
    let unique: u64 = NEXT_TEST_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
    let root: PathBuf = std::env::temp_dir().join(format!(
      "xrf-cli-verification-report-test-{}-{unique}",
      std::process::id()
    ));

    fs::create_dir_all(root.join("textures")).unwrap();
    fs::write(root.join("textures").join("a.dds"), []).unwrap();
    fs::write(root.join("textures").join("z.dds"), []).unwrap();

    root
  }

  #[test]
  fn reports_root_relative_paths_and_sorted_findings() {
    let root: PathBuf = temporary_gamedata_root();
    let mut report: GamedataVerificationReport = GamedataVerificationReport::with_duration(Duration::from_millis(42));

    report.add_check(
      GamedataVerificationType::Textures,
      Ok(TestCheckResult {
        duration: Duration::from_millis(7),
        findings: vec![
          Finding::new(
            RuleId::new("textures.dds").expect("Expected a non-empty test rule ID"),
            Some(root.join("textures").join("z.dds").display().to_string()),
            "Second finding",
          ),
          Finding::new(
            RuleId::new("textures.dds").expect("Expected a non-empty test rule ID"),
            Some(root.join("textures").join("a.dds").display().to_string()),
            "First finding",
          ),
        ],
      }),
    );

    let json: serde_json::Value =
      serde_json::to_value(GamedataVerificationReportPayload::new(&root, &report).build()).unwrap();

    fs::remove_dir_all(&root).unwrap();

    // The payload stays the command's own shape: unifying it across commands is deferred with 0050.
    assert!(json.get("schemaVersion").is_none());
    assert_eq!(json["status"], "failed");
    assert_eq!(json["duration"], 42);
    assert_eq!(json["checks"][0]["duration"], 7);
    assert_eq!(json["checks"][0]["verificationType"], "textures");
    assert_eq!(json["checks"][0]["findings"][0]["assetPath"], "textures/a.dds");
    assert_eq!(json["checks"][0]["findings"][1]["assetPath"], "textures/z.dds");
    assert_eq!(json["checks"][0]["findings"][1]["ruleId"], "textures.dds");
  }
}
