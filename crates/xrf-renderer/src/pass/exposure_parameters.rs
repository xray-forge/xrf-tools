use xrf_renderer_core::{GraphTexture, PassParameters, StorageValueMut, UniformBinding};

use crate::pass::exposure_state::ExposureState;
use crate::pass::exposure_uniform::ExposureUniform;

/// What the exposure's passes read and write: the frame's high part, the exposure's state, and how it adapts.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct ExposureParameters {
  #[texture(d2, unfilterable)]
  pub high: GraphTexture,
  #[storage]
  pub state: StorageValueMut<ExposureState>,
  #[uniform]
  pub params: UniformBinding<ExposureUniform>,
}
