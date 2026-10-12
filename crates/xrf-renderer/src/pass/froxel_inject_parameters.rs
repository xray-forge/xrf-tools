use xrf_renderer_core::{GraphTexture, PassParameters, StorageArray, UniformBinding};

use crate::pass::froxel_uniform::FroxelUniform;
use crate::pass::light_record::LightRecord;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::lights_uniform::LightsUniform;
use crate::pass::shadow_uniform::ShadowUniform;

/// What filling the froxels reads and writes: the frame's lighting, the sun's cascades, the local lights binned into
/// the view's clusters and their shadow atlas, the grid, the history the last frame wrote and this frame's.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct FroxelInjectParameters<'a> {
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[texture(d2_array, depth)]
  pub shadow_maps: GraphTexture,
  #[uniform]
  pub shadows: UniformBinding<ShadowUniform>,
  #[storage]
  pub records: StorageArray<LightRecord>,
  #[storage]
  pub counts: StorageArray<u32>,
  #[storage]
  pub items: StorageArray<u32>,
  #[uniform]
  pub lights: UniformBinding<LightsUniform>,
  #[texture(d2, depth)]
  pub shadow_atlas: GraphTexture,
  #[uniform]
  pub froxels: UniformBinding<FroxelUniform>,
  #[texture(d3, float)]
  pub froxel_history: GraphTexture,
  #[sampler(filtering)]
  pub froxel_sampler: &'a wgpu::Sampler,
  #[storage_texture(d3, rgba16float, write)]
  pub froxel_current: GraphTexture,
}
