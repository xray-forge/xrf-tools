use serde::Serialize;
use xrf_level::{LevelAiGraphCensus, LevelGeometryCensus};
use xrf_report::Report;

/// What `level verify` found and counted: its checks as every verify command reports them, beside its censuses; the AI
/// map's none where the level ships no map or it could not be read.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LevelVerifyReport<'a> {
  census: &'a LevelGeometryCensus,
  ai_census: Option<&'a LevelAiGraphCensus>,
  #[serde(flatten)]
  report: &'a Report,
}

impl<'a> LevelVerifyReport<'a> {
  /// The report of a sweep that counted `census` and `ai_census` and found `report`.
  pub(crate) fn new(
    census: &'a LevelGeometryCensus,
    ai_census: Option<&'a LevelAiGraphCensus>,
    report: &'a Report,
  ) -> Self {
    Self {
      census,
      ai_census,
      report,
    }
  }
}
