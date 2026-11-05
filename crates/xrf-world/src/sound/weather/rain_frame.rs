/// How hard it rains and blows, and how open the camera stands, as one frame gives the rain's loop.
#[derive(Clone, Copy, Debug, Default)]
pub struct RainFrame {
  /// The weather's `rain_density`, none while the rain is switched off or nothing plays.
  pub density: f32,
  /// The weather's `wind_velocity`, metres a second times a thousand as configs give it.
  pub wind_velocity: f32,
  /// The brightest of the camera's hemi cube faces but the one facing down; none where the level has no estimate,
  /// which counts as open sky.
  pub hemi: Option<f32>,
}
