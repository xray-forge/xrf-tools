use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::water_uniform::WaterUniform;

/// What the engine's water reads beside the scene: the frame's lighting and depth, both skies, and the nearest water.
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
}
