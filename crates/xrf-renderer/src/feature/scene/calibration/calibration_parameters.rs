use xrf_renderer_core::{PassParameters, UniformBinding};

use crate::feature::scene::calibration::calibration_uniform::CalibrationUniform;

/// What the calibration props are drawn with: where they stand and what they are.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub(crate) struct CalibrationParameters {
  #[uniform]
  pub calibration: UniformBinding<CalibrationUniform>,
}
