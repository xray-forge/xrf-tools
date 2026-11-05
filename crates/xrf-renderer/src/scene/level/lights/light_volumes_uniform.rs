use xrf_renderer_core::ShaderStruct;

use crate::scene::level::level_time::LevelTime;

/// What every light's shafts share: the noise they are broken by (`fx\fx_noise`), by its slot, and how far its two reads
/// have scrolled, each within the noise's repeat.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "LightVolumes")]
pub struct LightVolumesUniform {
  pub noise: u32,
  /// The first read's scroll across and the second's down.
  pub scroll: f32,
  pub scroll_down: f32,
}

impl LightVolumesUniform {
  /// The noise's scroll a second, the engine's global seconds over a hundred, and the second read's share of it.
  const SCROLL: f64 = 0.01;
  const SCROLL_DOWN: f64 = 0.70091;

  /// The noise in slot `noise`, scrolled to the level's clock `time`.
  pub fn new(noise: u32, time: LevelTime) -> Self {
    Self {
      noise,
      scroll: time.get_turns(Self::SCROLL),
      scroll_down: time.get_turns(Self::SCROLL * Self::SCROLL_DOWN),
    }
  }
}
