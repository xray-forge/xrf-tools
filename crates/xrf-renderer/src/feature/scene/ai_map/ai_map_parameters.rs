use xrf_renderer_core::{GraphTexture, PassParameters, StorageArray, UniformBinding};

use crate::feature::scene::ai_map::ai_map_uniform::AiMapUniform;

/// What the AI map's view reads: the frame's depth, the map's grid of cells and its nodes, and the field it shows.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub(crate) struct AiMapParameters {
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[storage]
  pub cells: StorageArray<u32>,
  #[storage]
  pub nodes: StorageArray<u32>,
  #[uniform]
  pub ai_map: UniformBinding<AiMapUniform>,
}
