use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::reflection_uniform::ReflectionUniform;

/// What a stage of the screen-space reflections reads: the G-buffer, its light, motion and occlusion, the material
/// table, the lighting and the settings, the pyramid of the nearest depth, the stage before's result and its rays'
/// lengths, and the last frame's accumulation and what it held.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct ReflectionParameters<'a> {
  #[texture(d2, unfilterable)]
  pub albedo_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub normal_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub material_target: GraphTexture,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub light_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub motion_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub occlusion_target: GraphTexture,
  #[texture(d3, float)]
  pub material_lut: GraphTexture,
  #[sampler(filtering)]
  pub lut_sampler: &'a wgpu::Sampler,
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[uniform]
  pub reflection: UniformBinding<ReflectionUniform>,
  #[texture(d2, unfilterable)]
  pub nearest_depth: GraphTexture,
  #[texture(d2, unfilterable)]
  pub traced: GraphTexture,
  #[texture(d2, unfilterable)]
  pub traced_length: GraphTexture,
  #[texture(d2, unfilterable)]
  pub history: GraphTexture,
  #[texture(d2, unfilterable)]
  pub history_held: GraphTexture,
}
