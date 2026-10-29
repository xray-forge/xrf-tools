use glam::Vec4;
use xrf_renderer_core::{GraphTexture, PassParameters, StorageArray, UniformBinding};

use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::thunder_uniform::ThunderUniform;

/// What one of a strike's draws reads: the strike, its texture, the bolt's model, empty for a glow, and the frame's
/// lighting, whose display the bolt's pictures enter the scene through.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct ThunderParameters<'a> {
  #[uniform]
  pub thunder: UniformBinding<ThunderUniform>,
  #[texture(d2, float)]
  pub thunder_texture: GraphTexture,
  #[sampler(filtering)]
  pub thunder_sampler: &'a wgpu::Sampler,
  #[storage]
  pub model_vertices: StorageArray<Vec4>,
  #[storage]
  pub model_indices: StorageArray<u32>,
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
}
