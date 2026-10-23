use serde::{Deserialize, Serialize};

/// What an envelope does before its first key or after its last, `BEH_*` (`xrCore/Animation/Envelope.hpp`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "u8", into = "u8")]
pub enum AnimationBehavior {
  /// `BEH_RESET`: zero.
  Reset,
  /// `BEH_CONSTANT`: the nearest end key's value.
  Constant,
  /// `BEH_REPEAT`: the keyed span over again.
  Repeat,
  /// `BEH_OSCILLATE`: the keyed span forwards and back.
  Oscillate,
  /// `BEH_OFFSET`: the keyed span over again, shifted by its rise each time.
  Offset,
  /// `BEH_LINEAR`: on along the end key's tangent.
  Linear,
  /// A value the engine has no name for, kept as it was read so an envelope written back says what it said. The engine
  /// evaluates such an envelope as if the time fell inside its keys.
  Unknown(u8),
}

impl Default for AnimationBehavior {
  /// `CEnvelope::CEnvelope` starts both ends at [`Self::Constant`].
  fn default() -> Self {
    Self::Constant
  }
}

impl From<u8> for AnimationBehavior {
  fn from(raw: u8) -> Self {
    match raw {
      0 => Self::Reset,
      1 => Self::Constant,
      2 => Self::Repeat,
      3 => Self::Oscillate,
      4 => Self::Offset,
      5 => Self::Linear,
      raw => Self::Unknown(raw),
    }
  }
}

impl From<AnimationBehavior> for u8 {
  fn from(behavior: AnimationBehavior) -> Self {
    match behavior {
      AnimationBehavior::Reset => 0,
      AnimationBehavior::Constant => 1,
      AnimationBehavior::Repeat => 2,
      AnimationBehavior::Oscillate => 3,
      AnimationBehavior::Offset => 4,
      AnimationBehavior::Linear => 5,
      AnimationBehavior::Unknown(raw) => raw,
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::AnimationBehavior;

  #[test]
  fn each_named_behavior_is_stored_as_the_number_the_engine_gives_it() {
    let named: [AnimationBehavior; 6] = [
      AnimationBehavior::Reset,
      AnimationBehavior::Constant,
      AnimationBehavior::Repeat,
      AnimationBehavior::Oscillate,
      AnimationBehavior::Offset,
      AnimationBehavior::Linear,
    ];

    for (raw, behavior) in (0_u8..).zip(named) {
      assert_eq!(AnimationBehavior::from(raw), behavior);
      assert_eq!(u8::from(behavior), raw);
    }
  }

  #[test]
  fn a_behavior_the_engine_does_not_name_keeps_its_number() {
    assert_eq!(AnimationBehavior::from(9), AnimationBehavior::Unknown(9));
    assert_eq!(u8::from(AnimationBehavior::Unknown(9)), 9);
  }
}
