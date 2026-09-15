use serde::Serialize;

use crate::plugins::archives::describe::omf::archive_omf_quantized::ArchiveOmfQuantized;

/// What the engine widens a blend by before using it, `fQuantizerRangeExt`
/// (`xrCore/Animation/SkeletonMotions.hpp`).
const RANGE_EXTENSION: f32 = 1.5;

/// How a motion blends in and out, in the values the engine blends with.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveOmfBlend {
  /// What the engine blends in over, `1.5 × Dequantize(accrue)`.
  pub accrue: f32,
  /// What the engine blends out over, after the rewrite.
  pub falloff: f32,
  pub declared_accrue: f32,
  pub declared_falloff: f32,
  /// Whether the engine replaced the declared falloff rather than reading it.
  pub is_falloff_replaced: bool,
}

impl ArchiveOmfBlend {
  /// The blend a definition produces, `CMotionDef::Load` (`xrCore/Animation/SkeletonMotions.cpp`).
  pub fn of(declared_accrue: f32, declared_falloff: f32, is_fx: bool) -> Self {
    let accrue: u16 = ArchiveOmfQuantized::to_steps(declared_accrue);
    let declared_steps: u16 = ArchiveOmfQuantized::to_steps(declared_falloff);
    let is_falloff_replaced: bool = !is_fx && declared_steps >= accrue;

    Self {
      accrue: Self::to_blend(accrue),
      falloff: Self::to_blend(if is_falloff_replaced {
        accrue.wrapping_sub(1)
      } else {
        declared_steps
      }),
      declared_accrue,
      declared_falloff,
      is_falloff_replaced,
    }
  }

  /// The widened value playback blends over.
  fn to_blend(steps: u16) -> f32 {
    RANGE_EXTENSION * ArchiveOmfQuantized::from_steps(steps)
  }
}

#[cfg(test)]
mod tests {
  use super::ArchiveOmfBlend;

  /// Close enough for a value that has been through a 1/655.35 quantizer twice.
  fn assert_near(value: f32, expected: f32) {
    assert!((value - expected).abs() < 0.01, "{value} is not near {expected}");
  }

  #[test]
  fn the_engine_blends_over_half_again_what_the_file_declares() {
    // The commonest pair in vanilla by a wide margin: accrue and falloff both 2.
    let blend: ArchiveOmfBlend = ArchiveOmfBlend::of(2.0, 2.0, false);

    assert_near(blend.accrue, 3.0);
    assert_near(blend.falloff, 3.0);
    assert_eq!(blend.declared_accrue, 2.0);
  }

  #[test]
  fn a_cycle_declaring_a_falloff_at_or_above_its_accrue_has_it_replaced() {
    let blend: ArchiveOmfBlend = ArchiveOmfBlend::of(2.0, 5.0, false);

    assert!(blend.is_falloff_replaced);
    assert_eq!(blend.declared_falloff, 5.0);
    assert!(blend.falloff < blend.accrue, "the replacement puts it one step under");
  }

  #[test]
  fn an_effect_keeps_the_falloff_it_declares() {
    let blend: ArchiveOmfBlend = ArchiveOmfBlend::of(2.0, 5.0, true);

    assert!(!blend.is_falloff_replaced);
    assert_near(blend.falloff, 7.5);
  }

  #[test]
  fn a_cycle_declaring_no_accrue_blends_out_over_the_quantizers_ceiling() {
    // 417 motions across the workspace trees, every one of them a cycle: `u16(0 - 1)` is 65535, not 0.
    let blend: ArchiveOmfBlend = ArchiveOmfBlend::of(0.0, 0.0, false);

    assert!(blend.is_falloff_replaced);
    assert_eq!(blend.accrue, 0.0);
    assert_near(blend.falloff, 150.0);
  }

  #[test]
  fn an_effect_declaring_no_accrue_is_left_alone() {
    let blend: ArchiveOmfBlend = ArchiveOmfBlend::of(0.0, 0.0, true);

    assert_eq!((blend.accrue, blend.falloff), (0.0, 0.0));
  }
}
