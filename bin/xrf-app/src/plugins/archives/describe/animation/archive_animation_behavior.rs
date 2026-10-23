use serde::Serialize;
use xrf_animation_envelope::AnimationBehavior;

/// What a channel does outside its keys, `BEH_*` (`xrCore/Animation/Envelope.hpp`).
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ArchiveAnimationBehavior {
  /// `BEH_RESET`: falls back to nothing.
  Reset,
  /// `BEH_CONSTANT`: holds the value of the nearest key, which is what everything shipped declares.
  Constant,
  /// `BEH_REPEAT`: plays the keyed range again from its start.
  Repeat,
  /// `BEH_OSCILLATE`: plays the keyed range again backwards.
  Oscillate,
  /// `BEH_OFFSET`: repeats, each pass starting where the last ended.
  Offset,
  /// `BEH_LINEAR`: carries on along the slope the end key leaves.
  Linear,
  /// A value the engine gives no name, kept as the number the file stores.
  Unnamed { value: u8 },
}

impl ArchiveAnimationBehavior {
  /// The behaviour an envelope declares.
  pub const fn of(behavior: AnimationBehavior) -> Self {
    match behavior {
      AnimationBehavior::Reset => Self::Reset,
      AnimationBehavior::Constant => Self::Constant,
      AnimationBehavior::Repeat => Self::Repeat,
      AnimationBehavior::Oscillate => Self::Oscillate,
      AnimationBehavior::Offset => Self::Offset,
      AnimationBehavior::Linear => Self::Linear,
      AnimationBehavior::Unknown(value) => Self::Unnamed { value },
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_animation_envelope::AnimationBehavior;

  use super::ArchiveAnimationBehavior;

  #[test]
  fn each_behavior_is_the_one_the_envelope_declares() {
    assert_eq!(
      ArchiveAnimationBehavior::of(AnimationBehavior::Reset),
      ArchiveAnimationBehavior::Reset
    );
    assert_eq!(
      ArchiveAnimationBehavior::of(AnimationBehavior::Linear),
      ArchiveAnimationBehavior::Linear
    );
  }

  #[test]
  fn a_value_the_engine_does_not_name_keeps_its_number() {
    assert_eq!(
      ArchiveAnimationBehavior::of(AnimationBehavior::from(9)),
      ArchiveAnimationBehavior::Unnamed { value: 9 }
    );
  }
}
