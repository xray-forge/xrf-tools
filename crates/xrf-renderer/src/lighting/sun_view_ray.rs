use glam::Vec3;

/// One edge of the view, from where it starts towards where it runs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SunViewRay {
  pub origin: Vec3,
  /// Normalized.
  pub direction: Vec3,
}
