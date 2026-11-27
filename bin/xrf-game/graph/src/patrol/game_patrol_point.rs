use xrf_math::glam::Vec3;

use crate::id::game_vertex_id::GameVertexId;

/// A point of a patrol path: where it stands on its level, the level node under it and the game vertex it belongs to.
#[derive(Clone, Debug, PartialEq)]
pub struct GamePatrolPoint {
  pub name: String,
  pub position: Vec3,
  pub level_vertex: u32,
  pub game_vertex: GameVertexId,
}
