use glam::Vec3;

/// The wind the weather's ambient effects stir, as the grass, the rain and the campfires read it: `wind_strength_factor`
/// and `wind_blast_direction`, in engine space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmbientGust {
  pub strength: f32,
  pub direction: Vec3,
}

impl AmbientGust {
  /// A campfire's idle particles' velocity, `wind_blast_direction * wind_strength_factor` (`CZoneCampfire`).
  pub fn get_velocity(&self) -> Vec3 {
    self.direction * self.strength
  }
}

impl Default for AmbientGust {
  /// Still air: the noise at no gusts stands at nothing, so the strength is a half, and the blast points along `x` as
  /// `CEnvironment` sets it up.
  fn default() -> Self {
    Self {
      strength: 0.5,
      direction: Vec3::X,
    }
  }
}
