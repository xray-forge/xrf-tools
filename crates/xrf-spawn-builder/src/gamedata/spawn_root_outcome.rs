use serde::Serialize;
use uuid::Uuid;
use xrf_report::Report;

/// What a root spawn build from a gamedata root wrote.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnRootOutcome {
  /// The spawn file written.
  pub path: String,
  /// The levels built, in id order.
  pub levels: Vec<String>,
  pub guid: Uuid,
  pub graph_guid: Uuid,
  pub objects: u32,
  pub artefact_points: u32,
  pub patrols: u32,
  pub report: Report,
}
