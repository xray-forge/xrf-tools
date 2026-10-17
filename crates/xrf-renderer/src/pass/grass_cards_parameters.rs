use xrf_renderer_core::{
  PassParameters, ShaderAtomicU32, StorageArray, StorageArrayMut, StorageValueMut, UniformBinding,
};

use crate::pass::grass_wind_uniform::GrassWindUniform;

/// What standing the grass's cards reads and writes (`grass/cards`): the models' arenas, the wind, and the cards'
/// corners, their triangles' records, how many stand and the cascades' draw of them.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 2)]
pub struct GrassCardsParameters {
  #[storage]
  pub positions: StorageArray<f32>,
  #[storage]
  pub uvs: StorageArray<f32>,
  #[storage]
  pub indices: StorageArray<u32>,
  #[uniform]
  pub wind: UniformBinding<GrassWindUniform>,
  #[storage]
  pub card_vertices: StorageArrayMut<u32>,
  #[storage]
  pub card_triangles: StorageArrayMut<u32>,
  #[storage]
  pub card_count: StorageValueMut<ShaderAtomicU32>,
  #[storage]
  pub card_draw: StorageValueMut<[u32; 4]>,
}
