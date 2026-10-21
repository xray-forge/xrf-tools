use xrf_renderer_core::{GraphTexture, PassParameters};

use crate::scene::texture::texture_cache::TextureCache;

/// What every pass drawing the weather's sky binds: both keyframes' sky cubes, their irradiance cubes and their clouds,
/// the two samplers they are read through, the cubes environment-mapped models mix toward, the sun's sprite, and the
/// sky as drawn prefiltered by roughness into a cube's mips (`SkyPrefilter`), indexed by a renderer-space direction,
/// and the same of the sky alone, nothing where the surroundings cover it (`SkyPrefilter::alone`).
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 2)]
pub struct SkyParameters<'a> {
  #[texture(cube, float)]
  pub sky_cube_0: GraphTexture,
  #[texture(cube, float)]
  pub sky_cube_1: GraphTexture,
  #[texture(cube, float)]
  pub sky_environment_0: GraphTexture,
  #[texture(cube, float)]
  pub sky_environment_1: GraphTexture,
  #[texture(d2, float)]
  pub sky_clouds_0: GraphTexture,
  #[texture(d2, float)]
  pub sky_clouds_1: GraphTexture,
  #[sampler(filtering)]
  pub sky_clamp: &'a wgpu::Sampler,
  #[sampler(filtering)]
  pub sky_repeat: &'a wgpu::Sampler,
  #[texture(cube, float)]
  pub environments: [GraphTexture; TextureCache::ENVIRONMENT_SLOTS as usize],
  #[texture(d2, float)]
  pub sky_sun: GraphTexture,
  #[texture(cube, float)]
  pub sky_prefiltered: GraphTexture,
  #[texture(cube, float)]
  pub sky_prefiltered_alone: GraphTexture,
}
