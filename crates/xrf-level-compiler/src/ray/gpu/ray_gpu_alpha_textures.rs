use crate::ray::gpu::ray_gpu_alpha_texture::RayGpuAlphaTexture;

/// The alpha textures as the GPU takes them ([`RayGpuAlphaTexture::pack`]): each one's record, and every texel's byte.
#[derive(Clone, Debug)]
pub(crate) struct RayGpuAlphaTextures {
  pub(crate) records: Vec<RayGpuAlphaTexture>,
  pub(crate) bytes: Vec<u8>,
}
