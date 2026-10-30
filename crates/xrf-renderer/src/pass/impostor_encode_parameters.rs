use glam::UVec4;
use xrf_renderer_core::{GraphTexture, PassParameters, StorageArrayMut, UniformBinding};

use crate::pass::impostor_encode_uniform::ImpostorEncodeUniform;

/// What compressing one level of a bake's three maps reads and writes: each map's level, the blocks it compresses
/// into, and where in them the level's blocks go.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct ImpostorEncodeParameters {
  #[binding(7)]
  #[texture(d2, unfilterable)]
  pub colour_source: GraphTexture,
  #[texture(d2, unfilterable)]
  pub normal_source: GraphTexture,
  #[texture(d2, unfilterable)]
  pub material_source: GraphTexture,
  #[storage]
  pub blocks: StorageArrayMut<UVec4>,
  #[uniform]
  pub settings: UniformBinding<ImpostorEncodeUniform>,
}
