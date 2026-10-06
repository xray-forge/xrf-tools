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
  /// The level's surface map's centre in `x` and `z`, its half width, and the height it is seen from; then its texels
  /// across and the metres it reaches down.
  pub surface: Vec4,
  pub surface_shape: Vec4,
  /// The level's wetness, how much of flat terrain puddles may cover, how much of a reflection a puddle takes at most,
  /// and how strongly rain ripples.
  pub puddles: Vec4,
  /// Metres out to which puddles are drawn, how full they are, how stormy the rain has been, and how soaked a long
  /// rain has left the level.
  pub puddle_state: Vec4,
}
