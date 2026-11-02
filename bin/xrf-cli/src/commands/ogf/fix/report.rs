use serde::Serialize;

use crate::commands::ogf::fix::ogf_fix_file_report::OgfFixFileReport;
use crate::commands::ogf::fix::ogf_fix_finding_report::OgfFixFindingReport;
use crate::commands::ogf::fix::ogf_fix_outcome::OgfFixOutcome;

/// What `ogf fix` did to the visuals it was pointed at.
///
/// Normalized visuals are `files` and failed ones `findings`, so neither count is carried beside its list; `unchanged`
/// is, since those visuals are not listed.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfFixReport {
  pub checked: usize,
  pub files: Vec<OgfFixFileReport>,
  pub findings: Vec<OgfFixFindingReport>,
  pub is_dry_run: bool,
  pub unchanged: usize,
}

impl OgfFixReport {
  pub fn new(outcomes: Vec<OgfFixOutcome>, is_dry_run: bool) -> Self {
    let mut report: Self = Self {
      checked: outcomes.len(),
      files: Vec::new(),
      findings: Vec::new(),
      is_dry_run,
      unchanged: 0,
    };

    for outcome in outcomes {
      match outcome {
        OgfFixOutcome::Unchanged => report.unchanged += 1,
        OgfFixOutcome::Normalized(file) => report.files.push(file),
        OgfFixOutcome::Failed(finding) => report.findings.push(finding),
      }
    }

    report
  }

  /// Total bytes discarded across every normalized visual.
  pub fn discarded_size(&self) -> usize {
    self.files.iter().map(OgfFixFileReport::discarded_size).sum()
  }
}
