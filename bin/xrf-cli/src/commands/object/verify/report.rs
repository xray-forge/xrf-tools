use serde::Serialize;
use xrf_rawdata::ObjectGeometryCensus;
use xrf_report::Report;

/// What `object verify` found and counted: its checks as every verify command reports them, beside its geometry census.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectVerifyReport<'a> {
  census: &'a ObjectGeometryCensus,
  #[serde(flatten)]
  report: &'a Report,
}

impl<'a> ObjectVerifyReport<'a> {
  /// The report of a verification that counted `census` and found `report`.
  pub fn new(census: &'a ObjectGeometryCensus, report: &'a Report) -> Self {
    Self { census, report }
  }
}
