use xrf_renderer_core::UniformBinding;

use crate::pass::wet_uniform::WetUniform;

/// What deciding the puddles reads beside the sites: the wet surfaces' settings and state, and the puddles' regional
/// noise and its sampler.
pub struct PuddleKeepInputs<'a> {
  pub wet: UniformBinding<WetUniform>,
  pub noise: &'a wgpu::TextureView,
  pub sampler: &'a wgpu::Sampler,
}
