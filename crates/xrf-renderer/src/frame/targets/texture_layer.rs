use xrf_renderer_core::GraphTexture;

/// One layer of a layered texture as a frame's graph knows it.
#[derive(Clone, Copy, Debug)]
pub struct TextureLayer {
  pub texture: GraphTexture,
  pub layer: u32,
}

impl TextureLayer {
  /// Its layer of `texture`, whole, as a copy reads or writes it.
  pub fn to_copy(texture: &wgpu::Texture, layer: u32) -> wgpu::TexelCopyTextureInfo<'_> {
    wgpu::TexelCopyTextureInfo {
      texture,
      mip_level: 0,
      origin: wgpu::Origin3d { x: 0, y: 0, z: layer },
      aspect: wgpu::TextureAspect::All,
    }
  }
}
