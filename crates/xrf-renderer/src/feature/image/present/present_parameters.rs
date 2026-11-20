use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::feature::image::present::present_uniform::PresentUniform;

/// What the present pass reads of a viewport's frame: the finished scene, its bloom, the marked models, and the
/// targets the debug views show.
#[derive(PassParameters)]
#[parameters(group = 1)]
pub(crate) struct PresentParameters<'a> {
  #[texture(d2, unfilterable)]
  pub scene: GraphTexture,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub albedo_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub normal_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub material_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub light_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub occlusion_target: GraphTexture,
  #[uniform]
  pub present: UniformBinding<PresentUniform>,
  /// The upscaled frame, or the scene again where none is.
  #[texture(d2, float)]
  pub upscaled: GraphTexture,
  #[texture(d2, unfilterable)]
  pub motion_target: GraphTexture,
  #[texture(d2, float)]
  pub bloom_target: GraphTexture,
  #[sampler(filtering)]
  pub bloom_sampler: &'a wgpu::Sampler,
  #[texture(d2, unfilterable)]
  pub indirect_light: GraphTexture,
  #[texture(d2, unfilterable)]
  pub reflections: GraphTexture,
  #[texture(d2, unfilterable)]
  pub reflection_ends: GraphTexture,
  /// At the drawn size, or a texel of none where no model is marked (`ViewTargets::MARKS`).
  #[texture(d2, unfilterable)]
  pub mark_target: GraphTexture,
}
