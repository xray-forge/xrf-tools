use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::fog_scattering_uniform::FogScatteringUniform;
use crate::pass::lighting_uniform::LightingUniform;

/// What a stage of the fog scattering reads: the image it blurs or scatters into, the blur a stage before it left, the
/// depth the fog is measured by, the lighting and the stage's own uniform.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct FogScatteringParameters<'a> {
  #[texture(d2, float)]
  pub source: GraphTexture,
  #[texture(d2, float)]
  pub blurred: GraphTexture,
  #[sampler(filtering)]
  pub source_sampler: &'a wgpu::Sampler,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[uniform]
  pub scattering: UniformBinding<FogScatteringUniform>,
}
