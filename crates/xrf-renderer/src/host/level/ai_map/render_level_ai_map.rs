use crate::host::level::ai_map::render_ai_map_node::RenderAiMapNode;

/// A level's AI map (`level.ai`) in engine coordinates: its grid and nodes, sorted by cell as the file stores them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderLevelAiMap {
  /// The grid's lowest x and z corner, where cell 0 lies.
  pub min_x: f32,
  pub min_z: f32,
  pub cell_size: f32,
  /// Cells along z (the packed cell's row length) and along x.
  pub row_length: u32,
  pub column_length: u32,
  /// Height of a packed height of 0, and the extent up to 65535.
  pub floor: f32,
  pub height: f32,
  pub nodes: Vec<RenderAiMapNode>,
}
