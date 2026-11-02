use serde::Serialize;
use xrf_level::LevelGeometryCensus;
use xrf_report::Report;

/// What `level verify` found and counted: its checks as every verify command reports them, beside its census.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelVerifyReport<'a> {
  census: &'a LevelGeometryCensus,
  #[serde(flatten)]
  report: &'a Report,
}

impl<'a> LevelVerifyReport<'a> {
  /// The report of a sweep that counted `census` and found `report`.
  pub fn new(census: &'a LevelGeometryCensus, report: &'a Report) -> Self {
    Self { census, report }
  }
}
