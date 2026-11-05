use xrf_renderer_core::ShaderStruct;

use crate::contract::feature::weather::water::render_water_settings::RenderWaterSettings;
use crate::scene::level::level_time::LevelTime;

/// What every water program reads as its `Water`: the clock, the settings both waters share, and the weather's
/// `water_intensity`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Water")]
pub struct WaterUniform {
  /// Seconds the water has moved, within `PERIOD`.
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
  /// Seconds the water had moved at the last frame, within `PERIOD`, where its wave stood then, which its motion is
  /// measured from.
  pub previous_time: f32,
}

impl WaterUniform {
  /// Seconds the water's clock repeats over: its wave runs on `sin(time)` and its normal layers turn about a circle at a
  /// tenth of it, which repeat over 2π and 20π; this spans the slower a hundred times, so a level's first hour and more
  /// runs as it always did. Its other readers, the reflection's march noise and jitter, follow no pattern a repeat
  /// breaks.
  pub const PERIOD: f64 = 2000.0 * std::f64::consts::PI;

  /// The settings at `time`, under the weather's `water_intensity`, its wave still since the last frame until it
  /// follows one.
  pub fn new(settings: &RenderWaterSettings, intensity: f32, time: LevelTime) -> Self {
    let time: f32 = time.get_phase(Self::PERIOD);

    Self {
      time,
      previous_time: time,
      wave_height: settings.wave_height,
      wave_speed: settings.wave_speed,
      ripple: settings.ripple,
      reflection: settings.reflection,
      intensity,
      soft: settings.is_soft as u32 as f32,
      distorted: settings.is_distorted as u32 as f32,
    }
  }

  /// The same water after a frame drawn at `previous`, which its wave's motion is measured from.
  pub fn following(self, previous: LevelTime) -> Self {
    Self {
      previous_time: previous.get_phase(Self::PERIOD),
      ..self
    }
  }
}
