use xrf_renderer_core::{
  GraphTexture, PassParameters, ShaderAtomicU32, StorageArrayMut, StorageValueMut, UniformBinding,
};

use crate::feature::image::exposure::exposure_state::ExposureState;
use crate::feature::image::exposure::histogram::exposure_histogram_uniform::ExposureHistogramUniform;
use crate::pass::frame_lighting::lighting_uniform::LightingUniform;

/// What the physical light's exposure reads and writes: the scene as drawn and its depth, the exposure's state, the
/// histogram's bins and how it meters, and the lighting, whose light unit and display the authored levels' engine meter
/// reads the frame by.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub(crate) struct ExposureHistogramParameters {
  #[texture(d2, unfilterable)]
  pub scene: GraphTexture,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[storage]
  pub state: StorageValueMut<ExposureState>,
  #[storage]
  pub bins: StorageArrayMut<ShaderAtomicU32>,
  #[uniform]
  pub params: UniformBinding<ExposureHistogramUniform>,
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
}
