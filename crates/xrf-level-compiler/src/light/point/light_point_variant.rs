use crate::ray::ray_rules::RayRules;

/// Which compiler's `LightPoint` to follow. The two copies differ in a static point light's colour, and trace their
/// shadow rays by their compiler's rules (`RayRules`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LightPointVariant {
  /// xrDO_Light's (`xrDO_Light/LightThread.cpp`): a static point light over its attenuation, its rays by
  /// `RayRules::DETAILS`.
  Details,
  /// xrLC's (`xrLC_Light/xrDeflectorLight.cpp`): a static point light times its attenuation's inverse less its falloff
  /// at that distance, so it ends at its range, its rays by `RayRules::LEVEL`.
  Level,
}

impl LightPointVariant {
  /// The rules its shadow rays follow.
  pub(crate) fn get_rules(self) -> RayRules {
    match self {
      Self::Details => RayRules::DETAILS,
      Self::Level => RayRules::LEVEL,
    }
  }
}
