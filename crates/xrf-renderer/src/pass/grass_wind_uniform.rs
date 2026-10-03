use glam::Vec4;

/// How the grass sways this frame, as `shaders/grass/grass.wgsl` reads it, in the engine's space: each wave's lean across
/// the ground, then each wave's direction with its phase in `w`, both over a turn.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GrassWindUniform {
  pub wind_1: Vec4,
  pub wind_2: Vec4,
  pub wave_1: Vec4,
  pub wave_2: Vec4,
  /// The same four the frame before, which a tuft's motion is measured from.
  pub previous_wind_1: Vec4,
  pub previous_wind_2: Vec4,
  pub previous_wave_1: Vec4,
  pub previous_wave_2: Vec4,
}

impl GrassWindUniform {
  /// This sway after the one before it, or after itself for a first frame.
  pub fn following(mut self, before: Option<&Self>) -> Self {
    let before: Self = before.copied().unwrap_or(self);

    self.previous_wind_1 = before.wind_1;
    self.previous_wind_2 = before.wind_2;
    self.previous_wave_1 = before.wave_1;
    self.previous_wave_2 = before.wave_2;
    self
  }
}
