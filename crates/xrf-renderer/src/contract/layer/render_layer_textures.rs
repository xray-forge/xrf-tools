use crate::contract::layer::render_layer_texture::RenderLayerTexture;
use crate::contract::layer::render_layer_texture_id::RenderLayerTextureId;

/// What a layer's textures become, in order: pixels set, then textures let go. Unlike a layer, never replaced by a later
/// one: each is applied in the order sent, as a glyph atlas grows by regions a later region builds on.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderLayerTextures {
  pub set: Vec<RenderLayerTexture>,
  pub free: Vec<RenderLayerTextureId>,
}
