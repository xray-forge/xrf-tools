use xrf_renderer_core::{
  GraphTexture, PassParameters, ShaderAtomicU32, StorageArrayMut, StorageValueMut, UniformBinding,
};

use crate::pass::exposure_histogram_uniform::ExposureHistogramUniform;
use crate::pass::exposure_state::ExposureState;

/// What the physical light's exposure reads and writes: the scene as drawn, the exposure's state, the histogram's bins
/// and how it meters.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct ExposureHistogramParameters {
  #[texture(d2, unfilterable)]
  pub scene: GraphTexture,
  #[storage]
  pub state: StorageValueMut<ExposureState>,
  #[storage]
  pub bins: StorageArrayMut<ShaderAtomicU32>,
  #[uniform]
  pub params: UniformBinding<ExposureHistogramUniform>,
}
