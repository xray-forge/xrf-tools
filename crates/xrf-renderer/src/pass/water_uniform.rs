use xrf_renderer_core::ShaderStruct;

use crate::contract::render_water_settings::RenderWaterSettings;

/// What every water program reads as its `Water`: the clock, the settings both waters share, and the weather's
/// `water_intensity`.
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
}

impl WaterUniform {
  /// The settings at `time`, under the weather's `water_intensity`.
  pub fn new(settings: &RenderWaterSettings, intensity: f32, time: f32) -> Self {
    Self {
      time,
      wave_height: settings.wave_height,
      wave_speed: settings.wave_speed,
      ripple: settings.ripple,
      reflection: settings.reflection,
      intensity,
      soft: settings.is_soft as u32 as f32,
      distorted: settings.is_distorted as u32 as f32,
    }
  }
}
