use xrf_renderer_core::{PassParameters, StorageValue, UniformBinding};

use crate::pass::flare_uniform::FlareUniform;
use crate::pass::lighting_uniform::LightingUniform;

/// What drawing the flares reads: the lens flare, how much of the sun shows as measured, the sampler, and the frame's
/// lighting, whose display the flares' pictures enter the scene through.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct FlareDrawParameters<'a> {
  #[uniform]
  pub flares: UniformBinding<FlareUniform>,
  #[storage]
  pub state: StorageValue<[f32; 4]>,
  #[sampler(filtering)]
  pub flare_sampler: &'a wgpu::Sampler,
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
}
