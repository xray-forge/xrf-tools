use crate::scene::texture::texture_cache::ENVIRONMENT_SLOTS;

/// The weather textures a frame's sky draws with, as the cache holds them when the frame is prepared, each its kind's
/// placeholder until its file is up: both keyframes' sky cubes, irradiance cubes and clouds, the sun's sprite, and the
/// environment cubes, the first standing for none.
pub struct SkyViews {
  pub cubes: [wgpu::TextureView; 2],
  pub irradiance: [wgpu::TextureView; 2],
  pub clouds: [wgpu::TextureView; 2],
  pub sun: wgpu::TextureView,
  pub environments: [wgpu::TextureView; ENVIRONMENT_SLOTS as usize],
}
