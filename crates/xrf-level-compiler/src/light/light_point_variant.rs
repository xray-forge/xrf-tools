use xrf_geometry::Facing;

/// Which compiler's `LightPoint` to follow. The two copies differ in a static point light's colour, in which side of
/// a face shadows, and in how much an alpha texel lets through.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightPointVariant {
  /// xrDO_Light's (`xrDO_Light/LightThread.cpp`): a static point light over its attenuation; a face shadows from its
  /// front only (`OPT_CULL`, `LightThread.cpp:292`); an alpha texel lets `1 - alpha` through.
  Details,
  /// xrLC's (`xrLC_Light/xrDeflectorLight.cpp:350-607`): a static point light times its attenuation's inverse less its
  /// falloff at that distance, so it ends at its range; a face shadows from either side (`ray_options(0)`); an alpha
  /// texel lets `1 - alpha²` through.
  Level,
}

impl LightPointVariant {
  /// The sides of a face its shadow rays meet.
  pub fn get_facing(self) -> Facing {
    match self {
      Self::Details => Facing::Front,
      Self::Level => Facing::Both,
    }
  }

  /// How much of a shadow ray passes a face stopping `opacity` of it.
  pub fn get_transmittance(self, opacity: f32) -> f32 {
    match self {
      Self::Details => 1.0 - opacity,
      Self::Level => 1.0 - opacity * opacity,
    }
  }
}
