use xrf_renderer_core::{GraphTexture, PassParameters, StorageValue, UniformBinding};

use crate::pass::enhanced_water_uniform::EnhancedWaterUniform;
use crate::pass::exposure_head::ExposureHead;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::water_uniform::WaterUniform;

/// What the enhanced water's reflection reads beside the scene: what every water program reads, the scene before the
/// water, the history the frames before accumulated, and the noise jittering its march; and what a traced hit is lit
/// and tonemapped by, the material table, the hemisphere's cubes and the exposure. Group 5, past the trace's two.
#[derive(PassParameters)]
#[parameters(group = 5)]
pub struct WaterReflectionParameters<'a> {
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[uniform]
  pub water: UniformBinding<WaterUniform>,
  #[uniform]
  pub enhanced: UniformBinding<EnhancedWaterUniform>,
  /// The G-buffer's depth, which the reflection is marched over.
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(cube, float)]
  pub sky_cube_0: GraphTexture,
  #[texture(cube, float)]
  pub sky_cube_1: GraphTexture,
  #[sampler(filtering)]
  pub sky_clamp: &'a wgpu::Sampler,
  /// The nearest water along each pixel, which the depth pass wrote: only that surface draws.
  #[texture(d2, depth)]
  pub nearest_water: GraphTexture,
  /// The scene as it stood before the water, which the reflection reads where its ray hit.
  #[texture(d2, float)]
  pub water_scene: GraphTexture,
  #[texture(d2, float)]
  pub reflection_history: GraphTexture,
  #[texture(d2, float)]
  pub blue_noise: GraphTexture,
  #[texture(d3, float)]
  pub material_lut: GraphTexture,
  #[sampler(filtering)]
  pub lut_sampler: &'a wgpu::Sampler,
  #[texture(cube, float)]
  pub sky_environment_0: GraphTexture,
  #[texture(cube, float)]
  pub sky_environment_1: GraphTexture,
  #[storage]
  pub exposure: StorageValue<ExposureHead>,
}
