use serde::Serialize;
use xrf_rawdata::ObjectGeometryCensus;
use xrf_report::Report;

/// What `object verify` found: its checks beside its geometry census.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ObjectVerifyReport<'a> {
  census: &'a ObjectGeometryCensus,
  #[serde(flatten)]
  report: &'a Report,
}

impl<'a> ObjectVerifyReport<'a> {
  /// The report of a verification that counted `census` and found `report`.
  pub(crate) fn new(census: &'a ObjectGeometryCensus, report: &'a Report) -> Self {
    Self { census, report }
  }
}
