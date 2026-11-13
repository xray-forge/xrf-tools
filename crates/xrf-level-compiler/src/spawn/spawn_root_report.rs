use std::time::Duration;

use xrf_report::{CheckId, CheckReport, Finding, RuleId, Status};

use crate::spawn::spawn_root_level::SpawnRootLevel;
use crate::spawn::spawn_separator_cut::SpawnSeparatorCut;
use crate::spawn::spawn_skipped_connection::SpawnSkippedConnection;

/// The checks of the root spawn build's report, each finding a warning naming its level; a build that refuses gives an
/// error instead.
pub(crate) struct SpawnRootReport;

impl SpawnRootReport {
  /// `spawn.graph-points`: the graph points each level's graph left out, as xrAI leaves them with a message.
  pub(crate) fn get_graph_points(levels: &[&SpawnRootLevel<'_>]) -> CheckReport {
    Self::new_check(
      "spawn.graph-points",
      None,
      levels
        .iter()
        .flat_map(|level| {
          level.removed.iter().map(|point| {
            Self::new_finding(
              "spawn.graph-point.removed",
              &level.merge.name,
              format!(
                "Graph point {} is left out of the game graph, {}",
                point.name, point.removal
              ),
            )
          })
        })
        .collect(),
    )
  }

  /// `spawn.connections`: the connections to levels outside the build the merge left out.
  pub(crate) fn get_connections(skipped: &[SpawnSkippedConnection]) -> CheckReport {
    Self::new_check(
      "spawn.connections",
      None,
      skipped
        .iter()
        .map(|connection| {
          Self::new_finding(
            "spawn.connection.skipped",
            &connection.level,
            format!(
              "Graph point {} connects to level {}, which is not built; the connection is left out",
              connection.point, connection.target_level
            ),
          )
        })
        .collect(),
    )
  }

  /// `spawn.separators`: the restrictors whose border cuts an AI map apart, the check having taken `took`.
  pub(crate) fn get_separators(cuts: &[SpawnSeparatorCut], took: Duration) -> CheckReport {
    Self::new_check(
      "spawn.separators",
      Some(took),
      cuts
        .iter()
        .map(|cut| Self::new_finding("spawn.separator.cut", &cut.level, Self::describe(cut)))
        .collect(),
    )
  }

  /// What a cut is, for a finding or a refusal.
  pub(crate) fn describe(cut: &SpawnSeparatorCut) -> String {
    let position = &cut.first_position;

    format!(
      "Restrictor {} cuts {} AI map nodes off, the first node {} at ({:.1}, {:.1}, {:.1})",
      cut.restrictor, cut.nodes, cut.first_node, position.x, position.y, position.z
    )
  }

  /// A check that passes whatever it finds: its findings are warnings.
  fn new_check(id: &'static str, took: Option<Duration>, findings: Vec<Finding>) -> CheckReport {
    CheckReport::new(CheckId::of_static(id), Status::Passed, took, findings)
  }

  fn new_finding(rule: &'static str, level: &str, message: String) -> Finding {
    Finding::new(RuleId::of_static(rule), Some(level.to_owned()), message)
  }
}
