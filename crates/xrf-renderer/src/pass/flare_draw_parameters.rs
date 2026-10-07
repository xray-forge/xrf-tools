use xrf_renderer_core::{PassParameters, StorageValue, UniformBinding};

use crate::pass::flare_uniform::FlareUniform;

/// What drawing the flares reads: the lens flare, how much of the sun shows as measured, and the sampler.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct FlareDrawParameters<'a> {
  #[uniform]
  pub flares: UniformBinding<FlareUniform>,
  #[storage]
  pub state: StorageValue<[f32; 4]>,
  #[sampler(filtering)]
  pub flare_sampler: &'a wgpu::Sampler,
}
