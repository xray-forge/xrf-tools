use crate::commands::ogf::fix::ogf_fix_file_report::OgfFixFileReport;
use crate::commands::ogf::fix::ogf_fix_finding_report::OgfFixFindingReport;

/// How one visual came out of the run.
pub enum OgfFixOutcome {
  Unchanged,
  Normalized(OgfFixFileReport),
  Failed(OgfFixFindingReport),
}
