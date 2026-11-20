use crate::ray::gpu::ray_gpu_alpha_textures::RayGpuAlphaTextures;
use crate::shadow::shadow_texture::ShadowTexture;

/// Where an alpha texture's texels lie among the uploaded bytes, and its size (`AlphaTexture`).
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct RayGpuAlphaTexture {
  offset: u32,
  width: u32,
  height: u32,
  _pad: u32,
}

impl RayGpuAlphaTexture {
  /// Each of `textures`' place among the bytes, and every texel's byte, the textures one after another, padded to a
  /// whole word.
  pub(crate) fn pack(textures: &[ShadowTexture]) -> RayGpuAlphaTextures {
    let mut bytes: Vec<u8> = Vec::new();
    let records: Vec<Self> = textures
      .iter()
      .map(|texture| {
        let record: Self = Self {
          offset: bytes.len() as u32,
          width: texture.alpha.width(),
          height: texture.alpha.height(),
          _pad: 0,
        };

        bytes.extend_from_slice(texture.alpha.as_raw());

        record
      })
      .collect();

    bytes.resize(bytes.len().next_multiple_of(4), 0);

    RayGpuAlphaTextures { records, bytes }
  }
}
