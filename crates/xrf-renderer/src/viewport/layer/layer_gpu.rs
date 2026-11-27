use std::collections::HashMap;

use crate::contract::layer::render_layer_texture_id::RenderLayerTextureId;
use crate::viewport::layer::layer_draw::LayerDraw;
use crate::viewport::layer::layer_texture::LayerTexture;

/// What of a viewport's layer is on the GPU, all of it made again from the layer's CPU side after a GPU is lost: its
/// textures, its meshes' buffers, and where each mesh sits in them.
#[derive(Default)]
pub(crate) struct LayerGpu {
  pub textures: HashMap<RenderLayerTextureId, LayerTexture>,
  pub vertices: Option<wgpu::Buffer>,
  pub indices: Option<wgpu::Buffer>,
  /// The uploaded layer's meshes, in order.
  pub draws: Vec<LayerDraw>,
}
