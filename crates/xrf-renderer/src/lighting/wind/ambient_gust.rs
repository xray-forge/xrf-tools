use glam::Vec3;

/// The wind the weather's ambient effects stir, as the grass, the rain and the campfires read it:
/// `wind_strength_factor` and `wind_blast_direction`, in engine space, and the gust of Monolith's rain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AmbientGust {
  pub strength: f32,
  pub direction: Vec3,
  /// Monolith's `Wind_Gust` (`CEffect_Rain::Prepare`): its rain's own noise, doubled, which its wind's velocity is
  /// gusted by before it leans the drops; within four thirds of nothing either way.
  pub rain: f32,
}

impl AmbientGust {
  /// A campfire's idle particles' velocity, `wind_blast_direction * wind_strength_factor` (`CZoneCampfire`).
  pub fn get_velocity(&self) -> Vec3 {
    self.direction * self.strength
  }
}

impl Default for AmbientGust {
  /// Still air: the noise at no gusts stands at nothing, so the strength is a half, and the blast points along `x` as
  /// `CEnvironment` sets it up; the rain's gust at nothing until its noise is first read.
  fn default() -> Self {
    Self {
      strength: 0.5,
      direction: Vec3::X,
      rain: 0.0,
    }
  }
}
