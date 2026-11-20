/// How the scene's walls shape one sound's way to the listener.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SourceAcoustics {
  /// The share heard unobstructed, from 0 to 1.
  pub(crate) occlusion: f32,
  /// What the walls in the way let through in each band, from 0 to 1.
  pub(crate) transmission: [f32; 3],
}

impl SourceAcoustics {
  /// Seconds the walls' hold on a sound takes to settle about two thirds of the way to a new simulation's: a few of the
  /// simulation's runs, so a sound moving past an edge, or between sampled points, swells and fades rather than steps.
  const EASE: f32 = 0.1;

  /// Moves `seconds` of the way toward `target`, by an exponential ease.
  pub(crate) fn approach(&mut self, target: Self, seconds: f32) {
    let share: f32 = 1.0 - (-seconds.max(0.0) / Self::EASE).exp();
    let ease = |from: f32, to: f32| from + (to - from) * share;

    self.occlusion = ease(self.occlusion, target.occlusion);
    self.transmission = std::array::from_fn(|band| ease(self.transmission[band], target.transmission[band]));
  }
}

impl Default for SourceAcoustics {
  /// Unobstructed.
  fn default() -> Self {
    Self {
      occlusion: 1.0,
      transmission: [1.0; 3],
    }
  }
}
