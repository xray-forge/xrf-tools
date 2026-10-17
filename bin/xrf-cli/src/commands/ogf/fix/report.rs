use serde::Serialize;

use crate::commands::ogf::fix::ogf_fix_file_report::OgfFixFileReport;
use crate::commands::ogf::fix::ogf_fix_finding_report::OgfFixFindingReport;
use crate::commands::ogf::fix::ogf_fix_outcome::OgfFixOutcome;

/// What `ogf fix` did to the visuals it was pointed at.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OgfFixReport {
  pub checked: usize,
  /// Total bytes discarded across every normalized visual.
  pub discarded_size: usize,
  pub failed: usize,
  pub files: Vec<OgfFixFileReport>,
  pub findings: Vec<OgfFixFindingReport>,
  pub is_dry_run: bool,
  pub normalized: usize,
  pub unchanged: usize,
}

impl OgfFixReport {
  pub fn new(outcomes: Vec<OgfFixOutcome>, is_dry_run: bool) -> Self {
    let mut report: Self = Self {
      checked: outcomes.len(),
      discarded_size: 0,
      failed: 0,
      files: Vec::new(),
      findings: Vec::new(),
      is_dry_run,
      normalized: 0,
      unchanged: 0,
    };

    for outcome in outcomes {
      match outcome {
        OgfFixOutcome::Unchanged => report.unchanged += 1,
        OgfFixOutcome::Normalized(file) => {
          report.normalized += 1;
          report.discarded_size += file.discarded_size;
          report.files.push(file);
        }
        OgfFixOutcome::Failed(finding) => {
          report.failed += 1;
          report.findings.push(finding);
        }
      }
    }

    report
  }
}
