use xrf_renderer_core::ShaderStruct;

use crate::contract::feature::weather::water::render_water_mode::RenderWaterMode;
use crate::contract::feature::weather::water::render_water_settings::RenderWaterSettings;
use crate::scene::level::level_time::LevelTime;

/// What every water program reads as its `Water`: the clock, the settings both waters share, and the weather's
/// `water_intensity`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Water")]
pub(crate) struct WaterUniform {
  /// Seconds the water has moved, within `PERIOD`.
  pub time: f32,
  /// `W_POSITION_SHIFT_HEIGHT` and `W_POSITION_SHIFT_SPEED`: the engine's water's wave; none for the enhanced water
  /// (`get_wave_height`).
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
  /// `time` at the last frame, which the wave's motion is measured from.
  pub previous_time: f32,
}

impl WaterUniform {
  /// Seconds the water's clock repeats over: a hundred of the normal layers' 20π turn, a whole number of the wave's 2π,
  /// so a level's first hour and more runs unchanged. The march noise and jitter it also drives show no repeat.
  pub(crate) const PERIOD: f64 = 2000.0 * std::f64::consts::PI;

  /// The settings at `time` under the weather's `water_intensity`, the wave still until it follows a last frame
  /// (`following`).
  pub(crate) fn new(settings: &RenderWaterSettings, intensity: f32, time: LevelTime) -> Self {
    let time: f32 = time.get_phase(Self::PERIOD);

    Self {
      time,
      previous_time: time,
      wave_height: Self::get_wave_height(settings),
      wave_speed: settings.wave_speed,
      ripple: settings.ripple,
      reflection: settings.reflection,
      intensity,
      soft: u32::from(settings.is_soft) as f32,
      distorted: u32::from(settings.is_distorted) as f32,
    }
  }

  /// How high the wave lifts the surface: the engine's water as the settings ask, the enhanced water not at all.
  ///
  /// The enhanced water's waves are its maps and parallax; lifting its geometry slides a grazing view's hit by tens of
  /// times the height, and a still view's reflection shimmered three to eighteen times as much under every
  /// antialiasing.
  fn get_wave_height(settings: &RenderWaterSettings) -> f32 {
    match settings.mode {
      RenderWaterMode::Engine => settings.wave_height,
      RenderWaterMode::Enhanced => 0.0,
    }
  }

  /// The same water after a frame drawn at `previous`, which its wave's motion is measured from.
  pub(crate) fn following(self, previous: LevelTime) -> Self {
    Self {
      previous_time: previous.get_phase(Self::PERIOD),
      ..self
    }
  }
}
