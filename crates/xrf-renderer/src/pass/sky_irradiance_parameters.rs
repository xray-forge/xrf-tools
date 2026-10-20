use xrf_renderer_core::{GraphTexture, PassParameters, StorageValue, StorageValueMut, UniformBinding};

use crate::pass::exposure_head::ExposureHead;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::sky_harmonics::SkyHarmonics;

/// What projecting the sky's irradiance reads and writes: both keyframes' sky cubes and clouds and their samplers, the
/// lighting that blends and tints the sky, the exposure whose scale the sky is drawn on, and the harmonics.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct SkyIrradianceParameters<'a> {
  #[texture(cube, float)]
  pub sky_cube_0: GraphTexture,
  #[texture(cube, float)]
  pub sky_cube_1: GraphTexture,
  #[texture(d2, float)]
  pub sky_clouds_0: GraphTexture,
  #[texture(d2, float)]
  pub sky_clouds_1: GraphTexture,
  #[sampler(filtering)]
  pub sky_clamp: &'a wgpu::Sampler,
  #[sampler(filtering)]
  pub sky_repeat: &'a wgpu::Sampler,
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[storage]
  pub exposure: StorageValue<ExposureHead>,
  #[storage]
  pub harmonics: StorageValueMut<SkyHarmonics>,
}
