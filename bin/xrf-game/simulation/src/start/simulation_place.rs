use xrf_game_graph::GameVertexId;
use xrf_math::glam::Vec3;

/// Where the simulation puts an object: a position on a level, the level node under it and its game vertex, as
/// `alife():create` takes them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SimulationPlace {
  pub position: Vec3,
  pub level_vertex: u32,
  pub game_vertex: GameVertexId,
}
