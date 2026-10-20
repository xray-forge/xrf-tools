use xrf_renderer_core::{PassParameters, UniformBinding};

use crate::pass::calibration_uniform::CalibrationUniform;

/// What the calibration props are drawn with: where they stand and what they are.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct CalibrationParameters {
  #[uniform]
  pub calibration: UniformBinding<CalibrationUniform>,
}
