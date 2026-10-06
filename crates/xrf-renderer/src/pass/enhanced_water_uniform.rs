use xrf_renderer_core::ShaderStruct;

use crate::contract::render_enhanced_water_settings::RenderEnhancedWaterSettings;
use crate::lighting::render_wind::RenderWind;
use crate::scene::level::water_flow::WaterFlow;

/// What the enhanced water's programs read as their `EnhancedWater`: its strengths, the weather's wind and rain, how
/// far its maps have scrolled, and whether its reflection drew.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "EnhancedWater")]
pub struct EnhancedWaterUniform {
  pub refraction: f32,
  pub turbidity: f32,
  pub soft_border: f32,
  pub reflectivity: f32,
  /// The reflection's blur and the noise mixing the clear reflection in.
  pub reflection_blur: f32,
  pub blur_noise: f32,
  /// One where the reflection drew this frame, and one where the history it accumulates over holds a frame.
  pub reflected: f32,
  pub history: f32,
  /// The weather's wind velocity, which strengthens the waves.
  pub wind_velocity: f32,
  /// The sun's highlight and the caustics.
  pub specular: f32,
  pub caustics: f32,
  /// The waves' height and the rain's ripples, and how hard it rains.
  pub parallax_height: f32,
  pub ripples: f32,
  pub rain: f32,
  /// How far the maps have scrolled: seconds at the flow, then at the normals', the heights' and the wind layer's
  /// paces.
  pub flowed: f32,
  pub waves: f32,
  pub heights: f32,
  pub gusts_x: f32,
  pub gusts_y: f32,
  /// How far the maps' repeat is broken.
  pub variation: f32,
}

impl EnhancedWaterUniform {
  /// `reflection` says whether its reflection draws this frame, and whether its history holds a frame.
  pub fn new(
    settings: &RenderEnhancedWaterSettings,
    (wind, rain): (RenderWind, f32),
    flow: WaterFlow,
    (reflected, history): (bool, bool),
  ) -> Self {
    Self {
      refraction: settings.refraction,
      turbidity: settings.turbidity,
      soft_border: settings.soft_border,
      reflectivity: settings.reflectivity,
      reflection_blur: settings.reflection_blur,
      blur_noise: settings.blur_noise,
      reflected: reflected as u32 as f32,
      history: history as u32 as f32,
      wind_velocity: wind.velocity,
      specular: settings.specular,
      caustics: settings.caustics,
      parallax_height: settings.parallax_height,
      ripples: settings.ripples,
      rain,
      flowed: flow.seconds,
      waves: flow.waves,
      heights: flow.heights,
      gusts_x: flow.gusts[0],
      gusts_y: flow.gusts[1],
      variation: settings.variation,
    }
  }
}
