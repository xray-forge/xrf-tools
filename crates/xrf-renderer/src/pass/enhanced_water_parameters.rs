use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::enhanced_water_uniform::EnhancedWaterUniform;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::water_uniform::WaterUniform;

/// What the enhanced water's surface reads beside the scene: what the engine's reads, the scene before the water, its
/// reflection blurred and as drawn, the light laid on the scene, and its maps.
#[derive(PassParameters)]
#[parameters(group = 3)]
pub struct EnhancedWaterParameters<'a> {
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[uniform]
  pub water: UniformBinding<WaterUniform>,
  #[uniform]
  pub enhanced: UniformBinding<EnhancedWaterUniform>,
  /// The G-buffer's depth, which the water is tested against and clouds by.
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
  /// The scene as it stood before the water, which it refracts.
  #[texture(d2, float)]
  pub water_scene: GraphTexture,
  /// Its reflection blurred at half the size, then as drawn, and the noise mixing the two.
  #[texture(d2, float)]
  pub reflection_blurred: GraphTexture,
  #[texture(d2, float)]
  pub reflection_clear: GraphTexture,
  #[texture(d2, float)]
  pub perlin_map: GraphTexture,
  /// What the lights laid on the scene, which says where the sun reaches the bottom.
  #[texture(d2, float)]
  pub light_target: GraphTexture,
  /// Its waves' normals, the wind's layer over them, the light it gathers on its bottom, its waves' height and the
  /// rain's ripples.
  #[texture(d2, float)]
  pub wave_map: GraphTexture,
  #[texture(d2, float)]
  pub wind_map: GraphTexture,
  #[texture(d2, float)]
  pub caustics_map: GraphTexture,
  #[texture(d2, float)]
  pub height_map: GraphTexture,
  #[texture(d2, float)]
  pub ripple_map: GraphTexture,
}
