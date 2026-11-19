use uuid::Uuid;
use xrf_math::Vector3d;
use xrf_spawn::GraphCrossTableCell;

use crate::graph::spawn_graph_point::SpawnGraphPoint;
use crate::graph::spawn_level_game_graph::SpawnLevelGameGraph;
use crate::map::spawn_ai_map::SpawnAiMap;

/// One level as the game graph merge takes it: its `game_levels.ltx` entry, AI map, graph points, game graph and
/// cross table.
#[derive(Clone)]
pub(crate) struct SpawnMergeLevel<'a> {
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
  /// The level's GUID, from its `level.ai`.
  pub(crate) fn get_guid(&self) -> Uuid {
    self.map.get_header().guid
  }
}
