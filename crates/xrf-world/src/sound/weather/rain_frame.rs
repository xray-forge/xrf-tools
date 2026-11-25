/// How hard it rains and blows, and how open the camera stands, as one frame gives the rain's loop.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct RainFrame {
  /// The weather's `rain_density`, none while the rain is switched off or nothing plays.
  pub density: f32,
  /// The share of its strongest the weather's wind blows at.
  pub wind_strength: f32,
  /// The brightest of the camera's hemi cube faces but the one facing down; none where the level has no estimate,
  /// which counts as open sky.
  pub hemi: Option<f32>,
}
