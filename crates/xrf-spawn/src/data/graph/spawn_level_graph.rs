use xrf_math::Vector3d;

use crate::data::graph::graph_level::GraphLevel;
use crate::data::patrols::patrol::Patrol;

/// What a spawn's graph and patrols hold of one level: every level of the graph with its place in the world, the game
/// vertices the level's graph points were compiled into, and the patrols that start on it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpawnLevelGraph {
  /// Every level of the graph, each with its offset in the world the levels share.
  pub levels: Vec<GraphLevel>,
  /// The level's game vertices, each by its id and where it stands on the level, engine space.
  pub points: Vec<(u16, Vector3d)>,
  /// The patrols whose first point stands on the level, in the spawn's order.
  pub patrols: Vec<Patrol>,
}

impl SpawnLevelGraph {
  /// The level of the graph named `name`, case aside.
  pub fn get_level(&self, name: &str) -> Option<&GraphLevel> {
    self.levels.iter().find(|level| level.name.eq_ignore_ascii_case(name))
  }
}
