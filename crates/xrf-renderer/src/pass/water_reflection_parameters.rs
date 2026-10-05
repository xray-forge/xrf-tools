use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::water_uniform::WaterUniform;

/// What the enhanced water's reflection reads beside the scene: what the surface reads of the frame, the history the
/// frames before accumulated, and the noise jittering its march.
#[derive(PassParameters)]
#[parameters(group = 3)]
pub struct WaterReflectionParameters<'a> {
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
  /// The history the reflection accumulates over, and the noise jittering its march. Eight to ten are the surface's.
  #[binding(11)]
  #[texture(d2, float)]
  pub reflection_history: GraphTexture,
  #[texture(d2, float)]
  pub blue_noise: GraphTexture,
}
