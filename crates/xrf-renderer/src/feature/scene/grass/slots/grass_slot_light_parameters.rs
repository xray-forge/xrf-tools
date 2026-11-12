use xrf_renderer_core::{GraphTexture, PassParameters, StorageArray, UniformBinding};

use crate::feature::scene::grass::slots::grass_slot_light_uniform::GrassSlotLightUniform;

/// What the slots' light view reads: the frame's depth, the level's slot grid and planted slots' records, and the
/// value it shows.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub(crate) struct GrassSlotLightParameters {
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[storage]
  pub grid: StorageArray<u32>,
  #[storage]
  pub slot_records: StorageArray<u32>,
  #[uniform]
  pub slot_light: UniformBinding<GrassSlotLightUniform>,
}
