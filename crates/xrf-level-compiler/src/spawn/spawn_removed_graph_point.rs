use crate::spawn::spawn_graph_point_removal::SpawnGraphPointRemoval;

/// A graph point the spawn build leaves out of the game graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpawnRemovedGraphPoint {
  pub name: String,
  pub removal: SpawnGraphPointRemoval,
}
