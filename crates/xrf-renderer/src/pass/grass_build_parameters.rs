use glam::Vec4;
use xrf_renderer_core::{
  GraphTexture, PassParameters, ShaderAtomicU32, StorageArrayMut, StorageValueMut, UniformBinding,
};

use crate::pass::grass_puddles_uniform::GrassPuddlesUniform;
use crate::pass::grass_uniform::GrassUniform;

/// A view's planting (`GrassBuild`): the ring (a key a cell, its ground's shape, two vectors a cached tuft), the stale
/// cells a band counts and the schedule, the frame's items as planted, each one's model and the same sorted by model;
/// what the view plants with; and this frame's puddle sites and where they stand, which no tuft is planted in.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct GrassBuildParameters {
  #[storage]
  pub keys: StorageArrayMut<u32>,
  #[storage]
  pub shapes: StorageArrayMut<Vec4>,
  #[storage]
  pub cached: StorageArrayMut<Vec4>,
  #[storage]
  pub band_counts: StorageArrayMut<ShaderAtomicU32>,
  #[storage]
  pub schedule: StorageValueMut<[ShaderAtomicU32; 3]>,
  #[storage]
  pub planted: StorageArrayMut<Vec4>,
  #[storage]
  pub planted_models: StorageArrayMut<u32>,
  #[storage]
  pub sorted: StorageArrayMut<Vec4>,
  #[uniform]
  pub params: UniformBinding<GrassUniform>,
  #[texture(d2, unfilterable)]
  pub puddles: GraphTexture,
  #[uniform]
  pub puddle_window: UniformBinding<GrassPuddlesUniform>,
}
