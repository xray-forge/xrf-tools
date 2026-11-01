use xrf_renderer_core::{PassParameters, ShaderAtomicU32, StorageArray, StorageArrayMut, UniformBinding};

use crate::feature::lighting::lights::lights_uniform::LightsUniform;
use crate::scene::level::lights::light_record::LightRecord;

/// What the light binning reads and writes: the lights' records, each cluster's count and its lights, and how the view
/// is cut.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct LightBinningParameters {
  #[storage]
  pub records: StorageArray<LightRecord>,
  #[storage]
  pub counts: StorageArrayMut<ShaderAtomicU32>,
  #[storage]
  pub items: StorageArrayMut<u32>,
  #[uniform]
  pub lights: UniformBinding<LightsUniform>,
}
