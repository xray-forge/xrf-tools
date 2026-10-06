use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::sky_prefilter_uniform::SkyPrefilterUniform;

/// What filling one mip of the prefiltered sky cube reads and writes: both keyframes' sky cubes and their sampler,
/// the mips filled before it (a texel of nothing for the first), the lighting that blends and tints the sky, the mip's
/// own settings, and the mip's six faces.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct SkyPrefilterParameters<'a> {
  #[texture(cube, float)]
  pub sky_cube_0: GraphTexture,
  #[texture(cube, float)]
  pub sky_cube_1: GraphTexture,
  #[sampler(filtering)]
  pub sky_clamp: &'a wgpu::Sampler,
  #[texture(cube, float)]
  pub source: GraphTexture,
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[uniform]
  pub settings: UniformBinding<SkyPrefilterUniform>,
  #[storage_texture(d2_array, rgba16float, write)]
  pub faces: GraphTexture,
}
