use crate::contract::render_water_settings::RenderWaterSettings;

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
  pub pad: f32,
}

impl WaterUniform {
  pub fn new(settings: &RenderWaterSettings, intensity: f32, time: f32) -> Self {
    Self {
      time,
      wave_height: settings.wave_height,
      wave_speed: settings.wave_speed,
      ripple: settings.ripple,
      reflection: settings.reflection,
      intensity,
      soft: settings.is_soft as u32 as f32,
      pad: 0.0,
    }
  }
}
