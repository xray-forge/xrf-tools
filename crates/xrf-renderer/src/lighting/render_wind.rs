/// The weather's wind, in the terms a keyframe uses: what leans the rain and drives the enhanced water's waves.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderWind {
  /// `wind_direction`, in radians, the way it blows in engine space.
  pub direction: f32,
  /// `wind_velocity`.
  pub velocity: f32,
}
