use crate::contract::layer::render_layer_texture_id::RenderLayerTextureId;
use crate::contract::layer::render_layer_vertex::RenderLayerVertex;
use crate::contract::viewport::render_rect::RenderRect;

/// A run of triangles the layer draws: clipped to a rectangle of the window, sampling one texture.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderLayerMesh {
  /// What of the window it may draw on, in device pixels.
  pub clip: RenderRect,
  pub texture: RenderLayerTextureId,
  pub vertices: Vec<RenderLayerVertex>,
  /// Three a triangle, into `vertices`.
  pub indices: Vec<u32>,
}
