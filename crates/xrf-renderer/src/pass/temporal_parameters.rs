use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::temporal_uniform::TemporalUniform;

/// What the temporal resolve reads: this frame as drawn, its depth, the history the last frame wrote and its sampler,
/// what moves the history onto this frame, and the motion drawn.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct TemporalParameters<'a> {
  #[texture(d2, unfilterable)]
  pub frame: GraphTexture,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, float)]
  pub history: GraphTexture,
  #[sampler(filtering)]
  pub history_sampler: &'a wgpu::Sampler,
  #[uniform]
  pub temporal: UniformBinding<TemporalUniform>,
  #[texture(d2, unfilterable)]
  pub motion_target: GraphTexture,
}
