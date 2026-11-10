use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::feature::lighting::contact_shadows::contact_shadow_uniform::ContactShadowUniform;
use crate::feature::lighting::shadows::shadow_uniform::ShadowUniform;
use crate::pass::frame_lighting::lighting_uniform::LightingUniform;

/// What the sun's pass reads: the G-buffer's albedo, normals, material and depth, the material table and the BRDF's
/// split-sum table, the frame's lighting, the sun's shadow cascades and what they were drawn with, the contact shadows
/// under them, and what the sun's traced shadow saw, a lit texel where it is not traced.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub(crate) struct SunParameters<'a> {
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
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[texture(d2_array, depth)]
  pub shadow_maps: GraphTexture,
  #[uniform]
  pub shadows: UniformBinding<ShadowUniform>,
  #[texture(d2, unfilterable)]
  pub contact_shadows: GraphTexture,
  /// How the contact shadows were marched: at what size, and from which texel of each two by two at half of it.
  #[uniform]
  pub contact: UniformBinding<ContactShadowUniform>,
  #[texture(d2, unfilterable)]
  pub sun_traced: GraphTexture,
  /// The wet surface and the gloss the wet reach added, which the anchored levels' surface takes its water film from.
  #[texture(d2, unfilterable)]
  pub wet_surface: GraphTexture,
  #[texture(d2, unfilterable)]
  pub wet_reach: GraphTexture,
  /// The comparison the cascades' maps are filtered through (`SunPass::get_compare`).
  #[sampler(comparison)]
  pub shadow_compare: &'a wgpu::Sampler,
}
