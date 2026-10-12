use xrf_renderer_core::{GraphBuffer, UniformBinding};

use crate::pass::lights_uniform::LightsUniform;

/// A frame's lights as its graph holds them: the records, the clusters' counts and items they are binned into, and the
/// uniform they are binned and drawn by.
#[derive(Clone, Copy)]
pub struct LightBuffers {
  pub records: GraphBuffer,
  pub counts: GraphBuffer,
  pub items: GraphBuffer,
  pub uniform: UniformBinding<LightsUniform>,
}
