use std::time::Duration;

use xrf_error::XrfError;
use xrf_report::{CheckId, CheckReport, Finding, RuleId, Status};
use xrf_spawn::SpawnSectionClasses;

use crate::merge::spawn_skipped_connection::SpawnSkippedConnection;
use crate::root::spawn_root_level::SpawnRootLevel;
use crate::separator::spawn_separator_cut::SpawnSeparatorCut;

/// The checks of the root spawn build's report, each finding a warning naming its level.
pub(crate) struct SpawnRootReport;

impl SpawnRootReport {
  /// `spawn.graph-points`: the graph points each level's graph left out, as xrAI does with a message.
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

  /// `spawn.configs`: the game's configs left unread, `unread` why, so each section's class came from the table alone.
  pub(crate) fn get_configs(unread: Option<&XrfError>) -> CheckReport {
    Self::new_check(
      "spawn.configs",
      None,
      unread
        .map(|error| {
          Finding::new(
            RuleId::of_static("spawn.configs.unread"),
            Some(String::from(SpawnSectionClasses::SYSTEM_LTX_PATH)),
            format!("{error}; each section's class comes from the built-in table of the games' sections"),
          )
        })
        .into_iter()
        .collect(),
    )
  }

  /// `spawn.connections`: the merge's skipped connections to levels outside the build.
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

  /// `spawn.separators`: the restrictors cutting an AI map apart, timed at `took`.
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

  /// A cut's message, for a finding or a refusal.
  pub(crate) fn describe(cut: &SpawnSeparatorCut) -> String {
    let position = &cut.first_position;

    format!(
      "Restrictor {} cuts {} AI map nodes off, the first node {} at ({:.1}, {:.1}, {:.1})",
      cut.restrictor, cut.nodes, cut.first_node, position.x, position.y, position.z
    )
  }

  /// A check that passes whatever it finds, so its findings are warnings.
  fn new_check(id: &'static str, took: Option<Duration>, findings: Vec<Finding>) -> CheckReport {
    CheckReport::new(CheckId::of_static(id), Status::Passed, took, findings)
  }

  fn new_finding(rule: &'static str, level: &str, message: String) -> Finding {
    Finding::new(RuleId::of_static(rule), Some(level.to_owned()), message)
  }
}
