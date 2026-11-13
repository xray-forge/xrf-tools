use xrf_math::Vector3d;

/// A level's graph point as a game vertex is built from it (`CGameGraphBuilder::load_graph_point`): where it stands, the
/// node under it, the location types it gives its vertex, and the point on another level it connects to, if any.
#[derive(Clone, Debug, PartialEq)]
pub struct SpawnGraphPoint {
  pub name: String,
  pub position: Vector3d<f32>,
  pub node: u32,
  pub locations: [u8; 4],
  /// The graph point on the other level a level changer leads through, empty for none.
  pub connection_point_name: String,
  pub connection_level_name: String,
}
