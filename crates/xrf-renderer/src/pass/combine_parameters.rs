use xrf_renderer_core::{GraphTexture, PassParameters, StorageValue, UniformBinding};

use crate::pass::exposure_head::ExposureHead;
use crate::pass::lighting_uniform::LightingUniform;

/// What the combine reads: the G-buffer and the light gathered over it, its specular split at the anchored levels, the
/// material table, the frame's lighting and exposure, the ambient occlusion, the sky behind the fog, the indirect light,
/// the reflections traced and how much of the sun reaches each pixel.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 1)]
pub struct CombineParameters<'a> {
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
  /// The coloured specular the sun and the lamps reflect at the anchored levels; a texel of nothing elsewhere.
  #[texture(d2, unfilterable)]
  pub specular_target: GraphTexture,
  #[texture(d3, float)]
  pub material_lut: GraphTexture,
  #[sampler(filtering)]
  pub lut_sampler: &'a wgpu::Sampler,
  #[uniform]
  pub lighting: UniformBinding<LightingUniform>,
  #[storage]
  pub exposure: StorageValue<ExposureHead>,
  #[texture(d2, unfilterable)]
  pub occlusion_target: GraphTexture,
  /// The sky the distance fog fades into, at half the frame's size.
  #[texture(d2, float)]
  pub sky_behind_map: GraphTexture,
  /// The sky's haze, which the far ground stands against under the horizon where the enhanced fog fades it.
  #[texture(d2, float)]
  pub haze_map: GraphTexture,
  #[texture(d2, unfilterable)]
  pub indirect_light: GraphTexture,
  #[texture(d2, unfilterable)]
  pub reflections: GraphTexture,
  #[texture(d2, unfilterable)]
  pub wet_surface: GraphTexture,
  /// The gloss the reflections' wet reach added beyond the engine's own wetting, which the lights' specular leaves out.
  #[texture(d2, unfilterable)]
  pub wet_reach: GraphTexture,
  /// How much of the sun reaches each pixel, which the sun's glint on a wet surface's water film is shadowed by.
  #[texture(d2, unfilterable)]
  pub sun_seen: GraphTexture,
}
