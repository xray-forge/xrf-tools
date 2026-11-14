use xrf_geometry::Facing;

use crate::ray::ray_alpha::RayAlpha;

/// The rules a shadow ray follows through the ray model, one source for the CPU's tracing and the GPU's: which sides of
/// a face stop it, how much an alpha texel lets through, whether it passes the face its point lies on, and whether a
/// face the GPU's ray test meets counts only where the CPU's (`Triangle::cross`) meets it too.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RayRules {
  pub(crate) facing: Facing,
  pub(crate) alpha: RayAlpha,
  /// Read by the GPU alone: the CPU's lighters pass a point's own face through their opacity.
  pub(crate) is_skipping: bool,
  /// Read by the GPU alone, whose ray test is not the CPU's.
  pub(crate) is_cpu_tested: bool,
}

impl RayRules {
  /// xrDO_Light's (`xrDO_Light/LightThread.cpp`): a face stops a ray from its front only (`OPT_CULL`), an alpha
  /// texel lets `1 - alpha` through, and no face is passed.
  pub(crate) const DETAILS: Self = Self {
    facing: Facing::Front,
    alpha: RayAlpha::Linear,
    is_skipping: false,
    is_cpu_tested: false,
  };

  /// xrAI's cover rays (`xrAI/compiler_cover.cpp`), which query the ray model as xrDO_Light does.
  pub(crate) const COVERS: Self = Self::DETAILS;

  /// xrLC's (`xrLC_Light/xrDeflectorLight.cpp`): a face stops a ray from either side (`ray_options(0)`), an alpha
  /// texel lets `1 - alpha²` through, and a ray passes its point's own face. The light baker's samples sit on their
  /// faces and its skies hold grazing directions: the device's watertight test takes crossings the CPU's determinant
  /// test turns away, a step of light apart on a lightmap, so the GPU tests each face again as the CPU does.
  pub(crate) const LEVEL: Self = Self {
    facing: Facing::Both,
    alpha: RayAlpha::Squared,
    is_skipping: true,
    is_cpu_tested: true,
  };

  /// The rules' bits, `RULE_*` in `shadow_rays.wgsl`: a face stops a ray from either side; an alpha texel lets
  /// `1 - alpha²` through rather than `1 - alpha`; a ray passes the face its skip code names; a face the device's ray
  /// test meets counts only where the CPU's meets it too.
  #[cfg(feature = "gpu")]
  const RULE_BOTH_SIDES: u32 = 1;
  #[cfg(feature = "gpu")]
  const RULE_SQUARED_ALPHA: u32 = 2;
  #[cfg(feature = "gpu")]
  const RULE_SKIPPING: u32 = 4;
  #[cfg(feature = "gpu")]
  const RULE_CPU_TEST: u32 = 8;

  /// How much of a shadow ray passes a face stopping `opacity` of it.
  pub(crate) fn get_transmittance(self, opacity: f32) -> f32 {
    match self.alpha {
      RayAlpha::Linear => 1.0 - opacity,
      RayAlpha::Squared => 1.0 - opacity * opacity,
    }
  }

  /// The rules as the shader reads them.
  #[cfg(feature = "gpu")]
  pub(crate) fn get_bits(self) -> u32 {
    let mut bits: u32 = 0;

    if self.facing == Facing::Both {
      bits |= Self::RULE_BOTH_SIDES;
    }

    if self.alpha == RayAlpha::Squared {
      bits |= Self::RULE_SQUARED_ALPHA;
    }

    if self.is_skipping {
      bits |= Self::RULE_SKIPPING;
    }

    if self.is_cpu_tested {
      bits |= Self::RULE_CPU_TEST;
    }

    bits
  }
}
