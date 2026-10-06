use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

/// What `shaders/common/wet.wgsl` declares as `Wet`: how hard it rains, the clock, the engine, the cover, and the
/// enhanced wetting's state.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Wet")]
pub struct WetUniform {
  /// `RainDensity.x`.
  pub density: f32,
  /// Seconds the rain has fallen.
  pub time: f32,
  /// One on Anomaly's engine.
  pub is_extended: f32,
  /// One where the surfaces are wetted as the enhanced rain wets them.
  pub is_enhanced: f32,
  /// The cover's centre in `x` and `z`, its half width, and the height it is seen from.
  pub window: Vec4,
  /// The level's wetness, how much of flat terrain puddles may cover, how much of a reflection a puddle takes at most,
  /// and how strongly rain ripples.
  pub puddles: Vec4,
}
