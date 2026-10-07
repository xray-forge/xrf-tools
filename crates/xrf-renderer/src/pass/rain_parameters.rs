use glam::Vec4;
use xrf_renderer_core::{GraphTexture, PassParameters, StorageArray, UniformBinding};

use crate::pass::rain_uniform::RainUniform;

/// What the rain's streaks and splashes read: the rain, its cover's depth, the streak's and the splash's textures, and
/// the splash's model.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct RainParameters<'a> {
  #[uniform]
  pub rain: UniformBinding<RainUniform>,
  #[texture(d2, depth)]
  pub cover: GraphTexture,
  #[texture(d2, float)]
  pub streak_texture: GraphTexture,
  #[texture(d2, float)]
  pub splash_texture: GraphTexture,
  #[sampler(filtering)]
  pub rain_sampler: &'a wgpu::Sampler,
  #[storage]
  pub splash_vertices: StorageArray<Vec4>,
  #[storage]
  pub splash_indices: StorageArray<u32>,
}
