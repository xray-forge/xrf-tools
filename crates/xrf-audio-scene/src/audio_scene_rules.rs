/// Whose voice allocation a scene follows: OpenXRay's (`xray-16/src/xrSound`) or Monolith's
/// (`xray-monolith/src/xrSound`). The acoustics are the same for both.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AudioSceneRules {
  #[default]
  Vanilla,
  Extended,
}

impl AudioSceneRules {
  /// `snd_targets` at its default: the voices realized at once, the rest simulated.
  pub fn get_targets(self) -> usize {
    match self {
      Self::Vanilla => 32,
      Self::Extended => 1024,
    }
  }
}
