use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::water_uniform::WaterUniform;

/// What the water's surface reads beside the scene: the frame's lighting, depth and light, both skies, the nearest
/// water, the scene before it, its reflection blurred and as drawn, and the enhanced water's maps.
#[derive(PassParameters)]
#[parameters(group = 3)]
pub struct WaterSurfaceParameters<'a> {
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[uniform]
  pub water: UniformBinding<WaterUniform>,
  /// The G-buffer's depth, which the water is tested against and fades by.
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
  /// The scene as it stood before the water, which the enhanced water refracts; a texel alone while the engine's draws.
  #[texture(d2, float)]
  pub water_scene: GraphTexture,
  /// The enhanced water's reflection blurred at half the size, then as drawn, and the noise mixing the two.
  #[texture(d2, float)]
  pub reflection_blurred: GraphTexture,
  #[texture(d2, float)]
  pub reflection_clear: GraphTexture,
  #[texture(d2, float)]
  pub perlin_map: GraphTexture,
  /// The enhanced water's waves' normals, the wind's layer over them, and the light it gathers on its bottom; then what
  /// the lights laid on the scene, which says where the sun reaches the bottom. Eleven and twelve are the reflection's.
  #[binding(13)]
  #[texture(d2, float)]
  pub wave_map: GraphTexture,
  #[texture(d2, float)]
  pub wind_map: GraphTexture,
  #[texture(d2, float)]
  pub caustics_map: GraphTexture,
  #[texture(d2, float)]
  pub light_target: GraphTexture,
  /// The enhanced water's waves' height, which its parallax marches into, and the rain's ripples on it.
  #[texture(d2, float)]
  pub height_map: GraphTexture,
  #[texture(d2, float)]
  pub ripple_map: GraphTexture,
}
