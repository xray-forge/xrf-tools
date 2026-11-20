use xrf_renderer_core::{PassParameters, StorageValue, UniformBinding};

use crate::feature::image::exposure::reading::exposure_head::ExposureHead;
use crate::pass::frame_lighting::lighting_uniform::LightingUniform;

/// What the haze map's sky reads beside the sky: the frame's lighting and exposure.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub(crate) struct SkyHazeParameters {
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[storage]
  pub exposure: StorageValue<ExposureHead>,
}
