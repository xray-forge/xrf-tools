use std::time::Duration;

use xrf_report::{CheckId, CheckReport, Finding, Report, Status};

use crate::verify::level_geometry_census::LevelGeometryCensus;

/// What a sweep has found so far, gathered so a check takes one accumulator rather than four.
#[derive(Debug, Default)]
pub(crate) struct LevelGeometryFindings {
  pub census: LevelGeometryCensus,
  pub read: Vec<Finding>,
  pub ranges: Vec<Finding>,
  pub geometry: Vec<Finding>,
}

impl LevelGeometryFindings {
  /// Turns what was gathered into the report a caller judges, one check per list.
  pub(crate) fn into_report(self, duration: Duration) -> (LevelGeometryCensus, Report) {
    (
      self.census,
      Report::new(vec![
        Self::check("read", self.read, duration),
        Self::check("ranges", self.ranges, duration),
        Self::check("geometry", self.geometry, duration),
      ]),
    )
  }

  /// The check `level.<id>` over `findings`, passed where it found nothing.
  fn check(id: &str, findings: Vec<Finding>, duration: Duration) -> CheckReport {
    CheckReport::new(
      CheckId::new(format!("level.{id}")).expect("Expected a non-empty check id"),
      Status::from_is_valid(findings.is_empty()),
      Some(duration),
      findings,
    )
  }
}
