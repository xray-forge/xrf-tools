use xrf_renderer_core::ShaderStruct;

use crate::scene::level::level_time::LevelTime;

/// The light shafts' shared noise (`fx\fx_noise`) slot and the scroll of its two reads, each wrapped to one repeat.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "LightVolumes")]
pub(crate) struct LightVolumesUniform {
  pub noise: u32,
  /// First read's scroll across, second read's scroll down.
  pub scroll: f32,
  pub scroll_down: f32,
}

impl LightVolumesUniform {
  /// Noise scroll per second (the engine's global seconds over a hundred), and the second read's share of it.
  const SCROLL: f64 = 0.01;
  const SCROLL_DOWN: f64 = 0.70091;

  /// The noise in slot `noise`, scrolled to level time `time`.
  pub(crate) fn new(noise: u32, time: LevelTime) -> Self {
    Self {
      noise,
      scroll: time.get_turns(Self::SCROLL),
      scroll_down: time.get_turns(Self::SCROLL * Self::SCROLL_DOWN),
    }
  }
}
