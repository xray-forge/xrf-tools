use xrf_renderer_core::{GraphTexture, PassParameters, StorageArray, UniformBinding};

use crate::pass::contact_shadow_uniform::ContactShadowUniform;
use crate::pass::light_record::LightRecord;
use crate::pass::lights_uniform::LightsUniform;

/// What the lights' pass reads: the G-buffer's normals, material and depth, the material table, the lights binned into
/// the view's clusters, the scene's light shadow atlas, and what the contact shadows towards the lights march by.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct LightsParameters<'a> {
  #[texture(d2, unfilterable)]
  pub normal_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub material_target: GraphTexture,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d3, float)]
  pub material_lut: GraphTexture,
  #[sampler(filtering)]
  pub lut_sampler: &'a wgpu::Sampler,
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
  pub contact: UniformBinding<ContactShadowUniform>,
}
