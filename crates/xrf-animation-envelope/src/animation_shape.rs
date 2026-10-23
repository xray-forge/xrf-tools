use serde::{Deserialize, Serialize};

/// The curve a key's span is drawn along, `SHAPE_*` (`xrCore/Animation/Envelope.hpp`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "u8", into = "u8")]
pub enum AnimationShape {
  /// `SHAPE_TCB`: a Kochanek-Bartels spline through the key.
  Tcb,
  /// `SHAPE_HERM`: a Hermite spline with the key's own tangents.
  Hermite,
  /// `SHAPE_BEZI`: a Bezier spline with the key's own tangents.
  Bezier,
  /// `SHAPE_LINE`: straight towards the key.
  Linear,
  /// `SHAPE_STEP`: held until the key, which then steps to its value rather than interpolating.
  Stepped,
  /// `SHAPE_BEZ2`: a Bezier spline whose handles are times and values.
  TimedBezier,
  /// A value the engine has no name for, kept as it was read so an envelope written back says what it said.
  Unknown(u8),
}

impl AnimationShape {
  /// Whether a key of this shape steps to its value, judged by the number it is stored as, which is how `st_Key::Save`
  /// and `st_Key::Load_2` decide whether its curve parameters are stored at all.
  pub fn is_stepped(self) -> bool {
    u8::from(self) == u8::from(Self::Stepped)
  }
}

impl Default for AnimationShape {
  /// `st_Key::st_Key` zeroes the key, which is [`Self::Tcb`].
  fn default() -> Self {
    Self::Tcb
  }
}

impl From<u8> for AnimationShape {
  fn from(raw: u8) -> Self {
    match raw {
      0 => Self::Tcb,
      1 => Self::Hermite,
      2 => Self::Bezier,
      3 => Self::Linear,
      4 => Self::Stepped,
      5 => Self::TimedBezier,
      raw => Self::Unknown(raw),
    }
  }
}

impl From<AnimationShape> for u8 {
  fn from(shape: AnimationShape) -> Self {
    match shape {
      AnimationShape::Tcb => 0,
      AnimationShape::Hermite => 1,
      AnimationShape::Bezier => 2,
      AnimationShape::Linear => 3,
      AnimationShape::Stepped => 4,
      AnimationShape::TimedBezier => 5,
      AnimationShape::Unknown(raw) => raw,
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::AnimationShape;

  #[test]
  fn each_named_shape_is_stored_as_the_number_the_engine_gives_it() {
    let named: [AnimationShape; 6] = [
      AnimationShape::Tcb,
      AnimationShape::Hermite,
      AnimationShape::Bezier,
      AnimationShape::Linear,
      AnimationShape::Stepped,
      AnimationShape::TimedBezier,
    ];

    for (raw, shape) in (0_u8..).zip(named) {
      assert_eq!(AnimationShape::from(raw), shape);
      assert_eq!(u8::from(shape), raw);
    }
  }

  #[test]
  fn a_shape_the_engine_does_not_name_keeps_its_number() {
    assert_eq!(AnimationShape::from(7), AnimationShape::Unknown(7));
    assert_eq!(u8::from(AnimationShape::Unknown(7)), 7);
  }

  #[test]
  fn a_shape_is_stepped_by_the_number_it_is_stored_as() {
    assert!(AnimationShape::Stepped.is_stepped());
    // Built by hand rather than read, but written as 4 all the same, so it must store no curve either.
    assert!(AnimationShape::Unknown(4).is_stepped());
    assert!(!AnimationShape::Tcb.is_stepped());
    assert!(!AnimationShape::Unknown(7).is_stepped());
  }
}
