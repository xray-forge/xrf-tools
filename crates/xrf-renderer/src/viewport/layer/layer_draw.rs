use std::ops::Range;

use crate::contract::layer::render_layer_texture_id::RenderLayerTextureId;
use crate::contract::viewport::render_rect::RenderRect;

/// One mesh of a layer as uploaded: its clip, its texture, and where its indices and vertices sit in the layer's
/// buffers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LayerDraw {
  pub clip: RenderRect,
  pub texture: RenderLayerTextureId,
  pub indices: Range<u32>,
  pub base_vertex: i32,
}
