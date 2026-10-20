/// The scale a view's light is written at, so a half float holds it however bright the scene: under the physical
/// light the exposure last read back from the GPU, one under the engine's; and the scale the last frame's light was
/// written at, by which a history kept from that frame is brought to this one's.
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
}

impl Default for PreExposure {
  fn default() -> Self {
    Self::NONE
  }
}
