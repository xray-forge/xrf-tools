use serde::Serialize;

/// Steps the engine's 16-bit quantizer divides a value into, `CMotionDef::Quantize`
/// (`xrCore/Animation/SkeletonMotions.hpp`).
const STEPS_PER_UNIT: f32 = 655.35;

/// The largest value the quantizer's `u16` admits, which is `65535 / 655.35` exactly.
const LARGEST_VALUE: f32 = 100.0;

/// A playback value as the engine reads it, beside the one the file stores.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveOmfQuantized {
  /// What the engine reads, after the quantizer.
  pub value: f32,
  /// The float the file stores.
  pub declared: f32,
  /// Whether the quantizer's range refused the declared value.
  pub is_clamped: bool,
}

impl ArchiveOmfQuantized {
  /// The value a stored float becomes on the way into `CMotionDef`.
  pub fn of(declared: f32) -> Self {
    Self {
      value: Self::from_steps(Self::to_steps(declared)),
      declared,
      is_clamped: !(0.0..=LARGEST_VALUE).contains(&declared),
    }
  }

  /// `CMotionDef::Quantize`: the step count a stored float lands on.
  pub(crate) fn to_steps(value: f32) -> u16 {
    let steps: f32 = (value * STEPS_PER_UNIT).floor();

    if steps.is_nan() {
      return 0;
    }

    steps.clamp(0.0, f32::from(u16::MAX)) as u16
  }

  /// `CMotionDef::Dequantize`: what a step count is worth.
  pub(crate) fn from_steps(steps: u16) -> f32 {
    f32::from(steps) / STEPS_PER_UNIT
  }
}

#[cfg(test)]
mod tests {
  use super::{ArchiveOmfQuantized, LARGEST_VALUE};

  /// What the quantizer's own step is worth, which is the most a round trip may lose.
  const STEP: f32 = 1.0 / 655.35;

  #[test]
  fn an_ordinary_value_survives_the_round_trip_within_one_step() {
    // 1.0 is the speed 4,757 of vanilla's 5,410 motions declare, and it does not survive exactly: the engine plays
    // them at 0.99947.
    let speed: ArchiveOmfQuantized = ArchiveOmfQuantized::of(1.0);

    assert!((speed.value - 1.0).abs() < STEP, "{} is not within a step", speed.value);
    assert_eq!(speed.declared, 1.0);
    assert!(
      !speed.is_clamped,
      "a step of rounding is not the range refusing a value"
    );
  }

  #[test]
  fn a_value_past_the_quantizers_range_is_reported_beside_what_the_engine_reads() {
    let power: ArchiveOmfQuantized = ArchiveOmfQuantized::of(150.0);

    assert_eq!((power.value, power.declared), (LARGEST_VALUE, 150.0));
    assert!(power.is_clamped);
  }

  #[test]
  fn a_negative_value_reads_as_zero_rather_than_wrapping() {
    let speed: ArchiveOmfQuantized = ArchiveOmfQuantized::of(-1.0);

    assert_eq!((speed.value, speed.declared), (0.0, -1.0));
    assert!(speed.is_clamped);
  }

  #[test]
  fn a_value_that_is_not_a_number_reads_as_zero() {
    let speed: ArchiveOmfQuantized = ArchiveOmfQuantized::of(f32::NAN);

    assert_eq!(speed.value, 0.0);
    assert!(speed.declared.is_nan());
    assert!(speed.is_clamped);
  }
}
