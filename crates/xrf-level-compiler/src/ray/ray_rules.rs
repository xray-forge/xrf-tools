use xrf_geometry::Facing;

use crate::ray::ray_alpha::RayAlpha;

/// The rules a shadow ray follows through the ray model, shared by the CPU's tracing and the GPU's: which sides of a
/// face stop it, how much an alpha texel lets through, whether it passes its point's own face, and whether a face the
/// GPU meets counts only where the CPU's test (`Triangle::cross`) meets it too.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RayRules {
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
  /// texel lets `1 - alpha²` through, and a ray passes its point's own face.
  ///
  /// The GPU retests each face as the CPU does: with samples on their faces and grazing sky directions, the device's
  /// watertight test takes crossings the CPU's determinant test turns away, a step of light apart on a lightmap.
  pub(crate) const LEVEL: Self = Self {
    facing: Facing::Both,
    alpha: RayAlpha::Squared,
    is_skipping: true,
    is_cpu_tested: true,
  };

  /// The rules' bits, `RULE_*` in `shadow_rays.wgsl`.
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
