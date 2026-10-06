use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::wet_uniform::WetUniform;

/// What deciding each puddle site's puddle this frame reads and writes: the sites, the puddles' regional noise and its
/// sampler, the wet surfaces' settings and state, and a puddle a cell.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct PuddleKeepParameters<'a> {
  #[binding(6)]
  #[texture(d2, unfilterable)]
  pub kept_sites: GraphTexture,
  #[texture(d2, float)]
  pub region_noise: GraphTexture,
  #[sampler(filtering)]
  pub noise_sampler: &'a wgpu::Sampler,
  #[uniform]
  pub wet: UniformBinding<WetUniform>,
  #[storage_texture(d2, rgba32float, write)]
  pub puddles: GraphTexture,
}
