use crate::host::level::ai_map::render_ai_map_node::RenderAiMapNode;

/// A level's AI map as its loader reads it (`level.ai`): the grid its nodes stand on and each node, sorted by its cell
/// as the file keeps them, in the engine's coordinates.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderLevelAiMap {
  /// The grid's lowest corner along x and z, where cell 0 stands.
  pub min_x: f32,
  pub min_z: f32,
  /// The side of a cell.
  pub cell_size: f32,
  /// Cells along z, which a packed cell counts x in rows of; and along x.
  pub row_length: u32,
  pub column_length: u32,
  /// The height a packed height of 0 stands at, and how far above it 65535 does.
  pub floor: f32,
  pub height: f32,
  pub nodes: Vec<RenderAiMapNode>,
}
