use xrf_renderer_core::{PassParameters, StorageValue, UniformBinding};

use crate::pass::exposure_head::ExposureHead;
use crate::pass::lighting_uniform::LightingUniform;

/// What the haze map's sky reads beside the sky: the frame's lighting and exposure.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct SkyHazeParameters {
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[storage]
  pub exposure: StorageValue<ExposureHead>,
}
