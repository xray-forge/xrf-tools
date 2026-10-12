use xrf_renderer_core::{GraphTexture, PassParameters, StorageValue, UniformBinding};

use crate::pass::exposure_head::ExposureHead;
use crate::pass::glass_uniform::GlassUniform;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::shadow_uniform::ShadowUniform;

/// What the enhanced glass reads beside its textures: the frame's lighting and exposure, the material table, the sun's
/// shadow cascades, its strengths, and the frame as it stood before the glass with the depth it stands on.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 3)]
pub struct GlassParameters<'a> {
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[storage]
  pub exposure: StorageValue<ExposureHead>,
  #[texture(d3, float)]
  pub material_lut: GraphTexture,
  #[sampler(filtering)]
  pub lut_sampler: &'a wgpu::Sampler,
  #[texture(d2_array, depth)]
  pub shadow_maps: GraphTexture,
  #[uniform]
  pub shadows: UniformBinding<ShadowUniform>,
  #[uniform]
  pub glass: UniformBinding<GlassUniform>,
  #[texture(d2, float)]
  pub glass_scene: GraphTexture,
  #[sampler(filtering)]
  pub scene_sampler: &'a wgpu::Sampler,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
}
