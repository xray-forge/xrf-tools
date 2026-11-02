use std::sync::Arc;

use crate::host::render_asset_source::RenderAssetSource;
use crate::scene::texture::texture_cache::TextureCache;

/// What putting a sector or a model into the static scene works with: the device and its queue, the encoder its copies
/// go into, and the cache its surfaces' textures are asked of, read from `source`.
pub struct StaticItemContext<'a> {
  pub device: &'a wgpu::Device,
  pub queue: &'a wgpu::Queue,
  pub encoder: &'a mut wgpu::CommandEncoder,
  pub textures: &'a mut TextureCache,
  pub source: &'a Arc<dyn RenderAssetSource>,
}
