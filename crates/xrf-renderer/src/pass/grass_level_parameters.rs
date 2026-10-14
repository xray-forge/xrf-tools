use xrf_renderer_core::{PassParameters, ShaderAtomicU32, StorageArray, StorageArrayMut, StorageValue};

use crate::pass::grass_model_record::GrassModelRecord;

/// The level's grass as its planting, cull and cards read it (`GrassLevel`): the slot grid, the planted slots' records,
/// their triangle bins and the triangles, the dither and the models; and what each frame counts into: the items planted
/// a model then the frame's total, where each model's range goes on filling, and one indexed draw a model.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct GrassLevelParameters {
  #[storage]
  pub grid: StorageArray<u32>,
  #[storage]
  pub slot_records: StorageArray<u32>,
  #[storage]
  pub bins: StorageArray<u32>,
  #[storage]
  pub triangles: StorageArray<f32>,
  #[storage]
  pub dither: StorageValue<[u32; 256]>,
  #[storage]
  pub models: StorageArray<GrassModelRecord>,
  #[storage]
  pub counts: StorageArrayMut<ShaderAtomicU32>,
  #[storage]
  pub cursors: StorageArrayMut<ShaderAtomicU32>,
  #[storage]
  pub args: StorageArrayMut<u32>,
}
