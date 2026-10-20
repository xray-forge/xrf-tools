use crate::lighting::light_unit::light_unit;

/// The scale a view's light is written at per cd/m², so a half float holds it however bright the scene: under the
/// physical light the exposure last read back from the GPU, one under the engine's; and the scale the last frame's light
/// was written at, by which a history kept from that frame is brought to this one's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreExposure {
  pub current: f32,
  pub previous: f32,
}

impl PreExposure {
  /// Light written as it is, as the engine's light always is.
  pub const NONE: Self = Self {
    current: 1.0,
    previous: 1.0,
  };

  /// The scale a light in the engine's units is written at this frame: its luminance (`light_unit`) at the
  /// pre-exposure.
  pub fn get_light_scale(self, is_physical: bool) -> f32 {
    light_unit(is_physical) * self.current
  }
}

impl Default for PreExposure {
  fn default() -> Self {
    Self::NONE
  }
}
