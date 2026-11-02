use std::path::Path;

use serde::Serialize;
use xrf_gamedata::GamedataVerificationCheckReport;
use xrf_report::{CheckReport, Finding};
use xrf_utils::to_portable_path_string;

/// One check of a `gamedata verify` run: the shared check shape, its findings naming assets relative to the verified
/// root, and the check's own one-line summary.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GamedataVerifyCheckReport {
  #[serde(flatten)]
  check: CheckReport,
  summary: String,
}

impl GamedataVerifyCheckReport {
  /// `check` as reported, its asset paths relative to `root` where they lie beneath it.
  pub fn new(root: &Path, check: &GamedataVerificationCheckReport) -> Self {
    let report: &CheckReport = check.get_report();
    let findings: Vec<Finding> = report
      .findings()
      .iter()
      .map(|finding| {
        Finding::new(
          finding.rule_id().clone(),
          finding.subject().map(|subject| {
            let path: &Path = Path::new(subject);

            to_portable_path_string(path.strip_prefix(root).unwrap_or(path))
          }),
          finding.message(),
        )
      })
      .collect();

    Self {
      check: CheckReport::new(report.id().clone(), report.status(), report.duration(), findings),
      summary: check.get_summary().to_string(),
    }
  }
}
