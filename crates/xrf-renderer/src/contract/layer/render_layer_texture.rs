use crate::contract::layer::render_layer_texture_id::RenderLayerTextureId;

/// Pixels a layer texture takes: the whole texture, made anew at their size, or a region of one it holds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderLayerTexture {
  pub id: RenderLayerTextureId,
  /// Where the pixels go in the texture, from its top left; none for a whole new texture of their size.
  pub origin: Option<[u32; 2]>,
  pub width: u32,
  pub height: u32,
  /// Eight-bit premultiplied RGBA, display encoded, row after row.
  pub pixels: Vec<u8>,
  /// Whether it is sampled nearest rather than linear.
  pub is_nearest: bool,
}
