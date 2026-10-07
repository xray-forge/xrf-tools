use xrf_renderer_core::{GraphTexture, PassParameters, StorageArray, UniformBinding};

use crate::pass::light_record::LightRecord;
use crate::pass::light_volume_record::LightVolumeRecord;
use crate::pass::light_volumes_uniform::LightVolumesUniform;

/// What the light volume pass reads: the shafts to draw, the lights they belong to, the scene's light shadow atlas and
/// the sampler comparing it, the noise breaking them, and the frame's depth its half-size depth is taken from.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct LightVolumeParameters<'a> {
  #[storage]
  pub light_volumes: StorageArray<LightVolumeRecord>,
  #[storage]
  pub records: StorageArray<LightRecord>,
  #[texture(d2, depth)]
  pub shadow_atlas: GraphTexture,
  #[sampler(comparison)]
  pub shadow_sampler: &'a wgpu::Sampler,
  #[uniform]
  pub volumes: UniformBinding<LightVolumesUniform>,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
}
