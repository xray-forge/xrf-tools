use xrf_renderer_core::ShaderStruct;

use crate::contract::render_water_settings::RenderWaterSettings;
use crate::lighting::render_wind::RenderWind;
use crate::scene::level::water_flow::WaterFlow;

/// What the water's shaders read as their `Water`: the clock, the settings, and the weather's `water_intensity`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Water")]
pub struct WaterUniform {
  /// Seconds the water has moved.
  pub time: f32,
  /// `W_POSITION_SHIFT_HEIGHT` and `W_POSITION_SHIFT_SPEED`.
  pub wave_height: f32,
  pub wave_speed: f32,
  /// What the normal layers' scroll and the sky's reflection are multiplied by.
  pub ripple: f32,
  pub reflection: f32,
  /// `water_intensity`.
  pub intensity: f32,
  /// One while soft water reads the depth behind it, `r2_soft_water`.
  pub soft: f32,
  /// One while the water writes the distortion it causes.
  pub distorted: f32,
  /// The enhanced water's refraction, turbidity and soft border.
  pub refraction: f32,
  pub turbidity: f32,
  pub soft_border: f32,
  pub reflectivity: f32,
  /// The enhanced water's reflection blur and the noise mixing the clear reflection in.
  pub reflection_blur: f32,
  pub blur_noise: f32,
  /// One where the reflection drew this frame, and one where the history it accumulates over holds a frame.
  pub reflected: f32,
  pub history: f32,
  /// The weather's wind velocity, which strengthens the enhanced water's waves.
  pub wind_velocity: f32,
  /// The enhanced water's sun highlight and caustics.
  pub specular: f32,
  pub caustics: f32,
  /// The enhanced water's wave height and rain ripples, and how hard it rains.
  pub parallax_height: f32,
  pub ripples: f32,
  pub rain: f32,
  /// How far the enhanced water's maps have scrolled: seconds at its flow, then at its normals', its heights' and its
  /// wind layer's paces.
  pub flowed: f32,
  pub waves: f32,
  pub heights: f32,
  pub gusts_x: f32,
  pub gusts_y: f32,
  /// How far the enhanced water breaks its maps' repeat.
  pub variation: f32,
}

impl WaterUniform {
  /// `time` is the clock and how far the enhanced water has flowed; `reflection` says whether its reflection draws
  /// this frame, and whether its history holds a frame.
  pub fn new(
    settings: &RenderWaterSettings,
    (intensity, wind, rain): (f32, RenderWind, f32),
    (time, flow): (f32, WaterFlow),
    (reflected, history): (bool, bool),
  ) -> Self {
    Self {
      time,
      wave_height: settings.wave_height,
      wave_speed: settings.wave_speed,
      ripple: settings.ripple,
      reflection: settings.reflection,
      intensity,
      soft: settings.is_soft as u32 as f32,
      distorted: settings.is_distorted as u32 as f32,
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
