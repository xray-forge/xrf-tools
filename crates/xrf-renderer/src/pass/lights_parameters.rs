use xrf_renderer_core::{GraphTexture, PassParameters, StorageArray, UniformBinding};

use crate::pass::contact_shadow_uniform::ContactShadowUniform;
use crate::pass::light_record::LightRecord;
use crate::pass::lights_uniform::LightsUniform;

/// What the lights' pass and the light visibility pass before it read: the G-buffer's albedo, normals, material and
/// depth, the material table and the BRDF's split-sum table, the lights binned into the view's clusters, and what the
/// contact shadows towards the lights march by.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct LightsParameters<'a> {
  #[texture(d2, unfilterable)]
  pub albedo_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub normal_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub material_target: GraphTexture,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d3, float)]
  pub material_lut: GraphTexture,
  #[texture(d2, unfilterable)]
  pub brdf_table: GraphTexture,
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
  #[uniform]
  pub contact: UniformBinding<ContactShadowUniform>,
  /// The wet surface and the gloss the wet reach added, which the anchored levels' surface takes its water film from.
  #[texture(d2, unfilterable)]
  pub wet_surface: GraphTexture,
  #[texture(d2, unfilterable)]
  pub wet_reach: GraphTexture,
}
