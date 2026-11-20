use xrf_math::Vector3d;

/// One game vertex of a level.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpawnLevelGraphPoint {
  /// The vertex's id in the game graph, its position in the graph's vertex list.
  pub game_vertex_id: u16,
  /// Where the vertex stands on its level, engine space.
  pub level_point: Vector3d,
}
