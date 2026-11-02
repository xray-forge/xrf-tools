/// Where bolts strike and how far a strike lights the scene, as `CEffect_Thunderbolt` loads them: angles in radians.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RenderThunderSettings {
  /// Above the horizon, the least and the most.
  pub altitude: [f32; 2],
  /// Either way of the heading opposite the sun.
  pub delta_longitude: f32,
  /// The nearest a bolt strikes, of the far plane.
  pub min_distance: f32,
  /// The most a bolt leans off the vertical.
  pub tilt: f32,
  /// The chance a strike is followed at once by another.
  pub second_probability: f32,
  /// How much of the strike's colour the sky, the sun and the fog are lit by.
  pub sky_color: f32,
  pub sun_color: f32,
  pub fog_color: f32,
}
