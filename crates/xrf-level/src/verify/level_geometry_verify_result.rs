use std::time::Duration;

use xrf_report::Report;

use crate::verify::level_geometry_census::LevelGeometryCensus;

/// What a sweep of a level's geometry concluded: the report a caller judges, and the counts behind it.
#[derive(Debug)]
pub struct LevelGeometryVerifyResult {
  pub census: LevelGeometryCensus,
  pub duration: Duration,
  pub report: Report,
}
