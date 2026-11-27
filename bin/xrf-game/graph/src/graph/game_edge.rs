use crate::id::game_vertex_id::GameVertexId;

/// An edge of the game graph: the vertex it leads to and how far, in metres along the ground the level graph measured.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GameEdge {
  pub to: GameVertexId,
  pub distance: f32,
}
