use xrf_renderer_core::{GraphTexture, PassParameters, StorageValue, UniformBinding};

use crate::pass::exposure_head::ExposureHead;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::surroundings_uniform::SurroundingsUniform;

/// What resolving one face of the surroundings cube reads and writes: the face's G-buffer as the static scene drew it,
/// the material table, both keyframes' sky and irradiance cubes and clouds and their samplers, the lighting, the
/// exposure whose scale the sky is drawn on, the face's settings, and the cube's first mip, whose face it writes.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct SurroundingsParameters<'a> {
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub albedo_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub normal_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub material_target: GraphTexture,
  #[texture(d3, float)]
  pub material_lut: GraphTexture,
  #[sampler(filtering)]
  pub lut_sampler: &'a wgpu::Sampler,
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
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[storage]
  pub exposure: StorageValue<ExposureHead>,
  #[uniform]
  pub settings: UniformBinding<SurroundingsUniform>,
  #[storage_texture(d2_array, rgba16float, write)]
  pub faces: GraphTexture,
}
