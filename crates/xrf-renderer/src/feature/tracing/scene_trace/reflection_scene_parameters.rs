use xrf_renderer_core::{GraphTexture, PassParameters, StorageValue, UniformBinding};

use crate::feature::image::exposure::reading::exposure_head::ExposureHead;
use crate::pass::frame_lighting::lighting_uniform::LightingUniform;

/// What the reflections' scene pass reads and writes, whichever way its rays come in (`SceneListParameters` or
/// `WaterRayParameters`).
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub(crate) struct ReflectionSceneParameters<'a> {
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[texture(d3, float)]
  pub material_lut: GraphTexture,
  #[sampler(filtering)]
  pub lut_sampler: &'a wgpu::Sampler,
  /// The exposure a water's hit is tonemapped at.
  #[storage]
  pub exposure: StorageValue<ExposureHead>,
  /// The surfaces' traced reflection, or the water's reflection.
  #[storage_texture(d2, rgba16float, write)]
  pub traced_out: GraphTexture,
  /// The surfaces' ray ends; unused for the water.
  #[storage_texture(d2, r32float, write)]
  pub ends_out: GraphTexture,
  /// The scene a hit the screen showed is read from: the last frame's for the surfaces, this frame's copied before
  /// the water for the water; likewise its depth.
  #[texture(d2, float)]
  pub screen_scene: GraphTexture,
  #[texture(d2, depth)]
  pub screen_depth: GraphTexture,
}
