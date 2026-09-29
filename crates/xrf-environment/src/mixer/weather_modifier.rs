use serde::Serialize;

/// One `level.env_mod` volume as the mixer weighs it, `CEnvModifier`: where it is, how far and how strongly it reaches,
/// and what it adds of each value its flags name.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherModifier {
  /// In engine space.
  pub position: [f32; 3],
  pub radius: f32,
  pub power: f32,
  pub far_plane: f32,
  pub fog_color: [f32; 3],
  pub fog_density: f32,
  pub ambient: [f32; 3],
  pub sky_color: [f32; 3],
  pub hemi_color: [f32; 3],
  /// `EEnvModUsedParams`: which values it adds to.
  pub flags: u16,
}

impl WeatherModifier {
  pub const FAR_PLANE: u16 = 1 << 0;
  pub const FOG_COLOR: u16 = 1 << 1;
  pub const FOG_DENSITY: u16 = 1 << 2;
  pub const AMBIENT_COLOR: u16 = 1 << 3;
  pub const SKY_COLOR: u16 = 1 << 4;
  pub const HEMI_COLOR: u16 = 1 << 5;
  /// Every value, what a file before the flag word carries.
  pub const ALL: u16 = u16::MAX;

  /// `CEnvModifier::sum`'s weight at a point: its power, falling off to nothing at its radius.
  pub fn get_power(&self, view: [f32; 3]) -> f32 {
    if self.is_out_of_reach(view) {
      return 0.0;
    }

    self.power * (1.0 - self.get_distance_sq(view).sqrt() / self.radius)
  }

  /// Whether a point is at or past its radius, where it adds nothing and flags nothing.
  pub fn is_out_of_reach(&self, view: [f32; 3]) -> bool {
    self.get_distance_sq(view) >= self.radius * self.radius
  }

  fn get_distance_sq(&self, view: [f32; 3]) -> f32 {
    (0..3).map(|axis| (view[axis] - self.position[axis]).powi(2)).sum()
  }
}
