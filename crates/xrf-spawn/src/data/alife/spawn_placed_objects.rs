use crate::data::alife::spawn_placed_object::SpawnPlacedObject;
use crate::data::alife::spawn_skipped_object::SpawnSkippedObject;
use crate::data::graph::graph_level::GraphLevel;

/// Every object a spawn places, on every level, each with the level it stands on, and what of the spawn could not be
/// read: what a new game is built from.
#[derive(Clone, Debug, PartialEq)]
pub struct SpawnPlacedObjects {
  /// The game graph's levels, which the objects' level ids name.
  pub levels: Vec<GraphLevel>,
  /// The objects read, in the spawn's order.
  pub objects: Vec<SpawnPlacedObject>,
  /// Objects the spawn declares.
  pub total: u32,
  /// Objects that could not be read.
  pub skipped: Vec<SpawnSkippedObject>,
}
