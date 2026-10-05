use crate::contract::render_water_settings::RenderWaterSettings;
use crate::lighting::render_wind::RenderWind;

/// What `shaders/static/water.wgsl` reads as its `Water`: the clock, the settings, and the weather's `water_intensity`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WaterUniform {
  pub time: f32,
  pub wave_height: f32,
  pub wave_speed: f32,
  pub ripple: f32,
  pub reflection: f32,
  pub intensity: f32,
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
  /// The weather's wind, which drives the enhanced water's waves: its direction in radians and its velocity.
  pub wind_direction: f32,
  pub wind_velocity: f32,
  /// The enhanced water's sun highlight and caustics.
  pub specular: f32,
  pub caustics: f32,
}

impl WaterUniform {
  /// `reflection` says whether the enhanced water's reflection draws this frame, and whether its history holds a frame.
  pub fn new(
    settings: &RenderWaterSettings,
    (intensity, wind): (f32, RenderWind),
    time: f32,
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
      wind_direction: wind.direction,
      wind_velocity: wind.velocity,
      specular: settings.specular,
      caustics: settings.caustics,
    }
  }
}
