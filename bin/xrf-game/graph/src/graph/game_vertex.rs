use xrf_math::glam::Vec3;

/// A vertex of the game graph (`CGameGraph::CVertex`): where it stands on its level and in the whole game's space, the
/// level node under it, its terrain kind, and its edges in the graph's edge list.
#[derive(Clone, Debug, PartialEq)]
pub struct GameVertex {
  /// The level it stands on, by the graph's level id.
  pub level: u8,
  /// The level graph node under it.
  pub level_vertex: u32,
  /// Where it stands in its level's space, where an object placed at it stands.
  pub level_point: Vec3,
  /// Where it stands in the space of every level laid out together, which straight-line distances between levels use.
  pub game_point: Vec3,
  /// Its terrain kind, four bytes a monster's terrain masks are matched against (`vertex_type`).
  pub kind: [u8; 4],
  /// Its edges: `edge_count` of the graph's edges from `first_edge`.
  pub first_edge: u32,
  pub edge_count: u8,
}
