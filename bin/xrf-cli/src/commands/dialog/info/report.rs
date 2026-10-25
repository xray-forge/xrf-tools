use serde::Serialize;
use xrf_report::Report;

use crate::commands::dialog::info::dialog_sweep_census::DialogSweepCensus;

/// What `dialog info` found and counted: its checks as every verify command reports them, beside its census.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogInfoReport<'a> {
  census: &'a DialogSweepCensus,
  #[serde(flatten)]
  report: &'a Report,
}

impl<'a> DialogInfoReport<'a> {
  /// The report of a sweep that counted `census` and found `report`.
  pub fn new(census: &'a DialogSweepCensus, report: &'a Report) -> Self {
    Self { census, report }
  }
}
