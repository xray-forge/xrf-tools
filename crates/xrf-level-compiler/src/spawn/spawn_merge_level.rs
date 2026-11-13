use uuid::Uuid;
use xrf_math::Vector3d;
use xrf_spawn::GraphCrossTableCell;

use crate::spawn::spawn_ai_map::SpawnAiMap;
use crate::spawn::spawn_graph_point::SpawnGraphPoint;
use crate::spawn::spawn_level_game_graph::SpawnLevelGameGraph;

/// One level as the game graph merge takes it: its `game_levels.ltx` entry, its AI map, and the graph points, game
/// graph and cross table built from them.
#[derive(Clone)]
pub struct SpawnMergeLevel<'a> {
  /// The level's `[levels]` section, `level<id>` in vanilla.
  pub section: String,
  pub name: String,
  /// The name a graph point names this level by in its connection, `caption`.
  pub caption: String,
  pub id: u8,
  /// Where the level stands in the world the levels share.
  pub offset: Vector3d<f32>,
  pub map: &'a SpawnAiMap,
  pub points: &'a [SpawnGraphPoint],
  pub graph: &'a SpawnLevelGameGraph,
  pub cells: &'a [GraphCrossTableCell],
}

impl SpawnMergeLevel<'_> {
  /// The level's identity, its `level.ai`'s.
  pub fn get_guid(&self) -> Uuid {
    self.map.get_header().guid
  }
}
