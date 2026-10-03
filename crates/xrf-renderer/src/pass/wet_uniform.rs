use glam::Vec4;

/// What `shaders/common/wet.wgsl` declares as `Wet`: how hard it rains, the clock, the engine and the cover.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WetUniform {
  /// `RainDensity.x`.
  pub density: f32,
  /// Seconds the rain has fallen.
  pub time: f32,
  /// One on Anomaly's engine.
  pub is_extended: f32,
  pub pad: f32,
  /// The cover's centre in `x` and `z`, its half width, and the height it is seen from.
  pub window: Vec4,
}
