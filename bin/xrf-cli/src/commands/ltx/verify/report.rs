use std::time::Duration;

use serde::Serialize;
use xrf_job::JobOutcome;
use xrf_ltx::{LtxFinding, LtxFindingKind, LtxProject, LtxProjectVerifyResult};
use xrf_report::{CheckId, CheckReport, Finding, Report, RuleId, Status};

/// What an `ltx verify` run reports to a machine: how much of the project it judged, and the shared report whose one
/// check carries every finding and the run's time.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxVerifyReport {
  checked_fields: usize,
  checked_sections: usize,
  invalid_sections: usize,
  skipped_sections: usize,
  /// How much of the run had passed when the per-file work began: mounting, indexing and assembling the project.
  #[serde(with = "xrf_utils::duration_ms")]
  startup_duration: Duration,
  total_files: usize,
  total_sections: usize,
  valid_sections: usize,
  #[serde(flatten)]
  report: Report,
}

impl LtxVerifyReport {
  /// The one check this command runs.
  const CHECK: &'static str = "ltx";

  /// `result` as the shared report, its findings named by the paths a person can act on in `project`.
  pub fn new(project: &LtxProject, result: &LtxProjectVerifyResult) -> Self {
    // A stopped run judged only the files it reached, which is no verdict on the rest.
    let status: Status = match result.outcome {
      JobOutcome::Cancelled => Status::Incomplete,
      JobOutcome::Completed => Status::from_is_valid(result.findings.is_empty()),
    };

    Self {
      checked_fields: result.checked_fields,
      checked_sections: result.checked_sections,
      invalid_sections: result.invalid_sections,
      skipped_sections: result.skipped_sections,
      startup_duration: result.startup_duration,
      total_files: result.total_files,
      total_sections: result.total_sections,
      valid_sections: result.valid_sections,
      report: Report::new(vec![CheckReport::new(
        CheckId::of_static(Self::CHECK),
        status,
        Some(result.duration),
        result
          .findings
          .iter()
          .map(|finding| Self::to_finding(project, finding))
          .collect(),
      )]),
    }
  }

  pub fn report(&self) -> &Report {
    &self.report
  }

  /// One finding as the shared report holds it: the config to open as its subject, the section and field it is about
  /// leading its message.
  fn to_finding(project: &LtxProject, finding: &LtxFinding) -> Finding {
    let reported: LtxFinding = project.with_reported_paths(finding);

    Finding::new(
      RuleId::of_static(Self::get_rule(finding.kind)),
      Some(reported.file.unwrap_or(reported.entry)),
      finding.get_located_message(),
    )
  }

  const fn get_rule(kind: LtxFindingKind) -> &'static str {
    match kind {
      LtxFindingKind::Parse => "ltx.parse",
      LtxFindingKind::Scheme => "ltx.scheme",
      LtxFindingKind::Dialect => "ltx.dialect",
      LtxFindingKind::Include => "ltx.include",
      LtxFindingKind::RootKeys => "ltx.root-keys",
      LtxFindingKind::Unreadable => "ltx.unreadable",
    }
  }
}
