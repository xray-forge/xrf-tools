/// How the scene's walls shape one sound's way to the listener.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SourceAcoustics {
  /// The share heard unobstructed, from 0 to 1.
  pub(crate) occlusion: f32,
  /// What the walls in the way let through in each band, from 0 to 1.
  pub(crate) transmission: [f32; 3],
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
