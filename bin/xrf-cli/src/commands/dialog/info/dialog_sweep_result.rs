use std::time::Duration;

use xrf_report::Report;

use crate::commands::dialog::info::dialog_sweep_census::DialogSweepCensus;

/// Outcome of a sweep: a finalized report and the counts behind it.
#[derive(Debug)]
pub struct DialogSweepResult {
  pub census: DialogSweepCensus,
  pub duration: Duration,
  pub report: Report,
}
