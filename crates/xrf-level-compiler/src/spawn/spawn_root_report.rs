use std::time::Duration;

use xrf_report::{CheckId, CheckReport, Finding, Report, RuleId, Status};

use crate::spawn::spawn_separator_cut::SpawnSeparatorCut;
use crate::spawn::spawn_skipped_connection::SpawnSkippedConnection;

/// The root spawn build's report: its findings are warnings, a build that refuses gives an error instead.
pub(crate) struct SpawnRootReport;

impl SpawnRootReport {
  /// The level connections the merge left out, and the restrictors whose border cuts an AI map apart, the separator
  /// check having taken `separators_took`.
  pub(crate) fn build(
    skipped: &[SpawnSkippedConnection],
    cuts: &[SpawnSeparatorCut],
    separators_took: Duration,
  ) -> Report {
    Report::new(vec![
      CheckReport::new(
        CheckId::of_static("spawn.connections"),
        Status::Passed,
        None,
        skipped
          .iter()
          .map(|connection| {
            Finding::new(
              RuleId::of_static("spawn.connection.skipped"),
              Some(connection.level.clone()),
              format!(
                "Graph point {} connects to level {}, which is not built; the connection is left out",
                connection.point, connection.target_level
              ),
            )
          })
          .collect(),
      ),
      CheckReport::new(
        CheckId::of_static("spawn.separators"),
        Status::Passed,
        Some(separators_took),
        cuts
          .iter()
          .map(|cut| {
            Finding::new(
              RuleId::of_static("spawn.separator.cut"),
              Some(cut.level.clone()),
              Self::describe(cut),
            )
          })
          .collect(),
      ),
    ])
  }

  /// What a cut is, for a finding or a refusal.
  pub(crate) fn describe(cut: &SpawnSeparatorCut) -> String {
    let position = &cut.first_position;

    format!(
      "Restrictor {} cuts {} AI map nodes off, the first node {} at ({:.1}, {:.1}, {:.1})",
      cut.restrictor, cut.nodes, cut.first_node, position.x, position.y, position.z
    )
  }
}
