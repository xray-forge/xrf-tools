use xrf_renderer_core::{GraphTexture, PassParameters, StorageValue, UniformBinding};

use crate::pass::exposure_head::ExposureHead;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::reflection_uniform::ReflectionUniform;

/// What a stage of the screen-space reflections reads: the G-buffer, its light, occlusion and motion, the wet surface,
/// the material table, the lighting, the exposure and the settings; the frame's nearest depth pyramid and the blue
/// noise rays are scattered by; the last frame's scene and depth a hit reads its colour from; the stages before (the
/// trace, the reprojected history and its variance, the eighth-size average, the prefiltered reflection); and the last
/// frame's reflection, surface and held distance, read through a filtering sampler.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct ReflectionParameters<'a> {
  #[texture(d2, unfilterable)]
  pub albedo_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub normal_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub material_target: GraphTexture,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub light_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub occlusion_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub wet_surface: GraphTexture,
  #[texture(d3, float)]
  pub material_lut: GraphTexture,
  #[sampler(filtering)]
  pub lut_sampler: &'a wgpu::Sampler,
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[uniform]
  pub reflection: UniformBinding<ReflectionUniform>,
  #[storage]
  pub exposure: StorageValue<ExposureHead>,
  #[texture(d2, unfilterable)]
  pub motion_target: GraphTexture,
  #[texture(d2, float)]
  pub source_scene: GraphTexture,
  #[texture(d2, depth)]
  pub source_depth: GraphTexture,
  #[texture(d2, depth)]
  pub traced_depth: GraphTexture,
  #[texture(d2, unfilterable)]
  pub nearest_depth: GraphTexture,
  #[texture(d2, unfilterable)]
  pub blue_noise: GraphTexture,
  #[texture(d2, unfilterable)]
  pub traced: GraphTexture,
  #[texture(d2, unfilterable)]
  pub reprojected: GraphTexture,
  #[texture(d2, unfilterable)]
  pub variance: GraphTexture,
  #[texture(d2, float)]
  pub average: GraphTexture,
  #[texture(d2, unfilterable)]
  pub prefiltered: GraphTexture,
  #[texture(d2, float)]
  pub history_radiance: GraphTexture,
  #[texture(d2, float)]
  pub history_surface: GraphTexture,
  #[texture(d2, unfilterable)]
  pub history_held: GraphTexture,
  /// This frame's resolve, which the upscale brings to the frame's size, and the last frame's full-size history.
  #[texture(d2, unfilterable)]
  pub resolved: GraphTexture,
  #[texture(d2, float)]
  pub history_full: GraphTexture,
  #[sampler(filtering)]
  pub history_sampler: &'a wgpu::Sampler,
}
