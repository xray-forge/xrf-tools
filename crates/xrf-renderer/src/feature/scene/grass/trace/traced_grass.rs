use xrf_renderer_core::{GraphBuffer, UniformBinding};

use crate::feature::scene::grass::trace::trace_grass_uniform::TraceGrassUniform;

/// What a traced hit reads of the grass this frame (`TraceParameters`' grass): the planting's slot grid, its slots, its
/// models and each model's colour averaged, which cover a terrain hit, the planting as the hit reads it, and the cards
/// the view stands near its eye, whose holes a ray passes; a buffer of nothing in place of what the frame holds none
/// of.
#[derive(Clone, Copy)]
pub struct TracedGrass {
  pub grid: GraphBuffer,
  pub slots: GraphBuffer,
  pub models: GraphBuffer,
  pub colours: GraphBuffer,
  pub uniform: UniformBinding<TraceGrassUniform>,
  pub cards: GraphBuffer,
}
