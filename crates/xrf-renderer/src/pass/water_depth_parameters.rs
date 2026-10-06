use xrf_renderer_core::{PassParameters, UniformBinding};

use crate::pass::water_uniform::WaterUniform;

/// What the water's depth pass reads beside the scene: the water's uniform, which lifts its waves.
#[derive(PassParameters)]
#[parameters(group = 3)]
pub struct WaterDepthParameters {
  #[binding(1)]
  #[uniform]
  pub water: UniformBinding<WaterUniform>,
}
