/// How a bone's joint lets it move against its parent, `EJointType`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkeletonJointKind {
  /// Welded to its parent.
  #[default]
  Rigid,
  Cloth,
  /// Turns within its three limits.
  Joint,
  /// Spins about one axis and steers about another.
  Wheel,
  /// No joint: the bone is free of its parent.
  None,
  /// Slides along an axis.
  Slider,
  /// A value the engine does not name, kept so it writes back as stored.
  Unknown(u32),
}

impl From<u32> for SkeletonJointKind {
  fn from(value: u32) -> Self {
    match value {
      0 => Self::Rigid,
      1 => Self::Cloth,
      2 => Self::Joint,
      3 => Self::Wheel,
      4 => Self::None,
      5 => Self::Slider,
      other => Self::Unknown(other),
    }
  }
}

impl From<SkeletonJointKind> for u32 {
  fn from(kind: SkeletonJointKind) -> Self {
    match kind {
      SkeletonJointKind::Rigid => 0,
      SkeletonJointKind::Cloth => 1,
      SkeletonJointKind::Joint => 2,
      SkeletonJointKind::Wheel => 3,
      SkeletonJointKind::None => 4,
      SkeletonJointKind::Slider => 5,
      SkeletonJointKind::Unknown(value) => value,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::SkeletonJointKind;

  #[test]
  fn names_the_joints_the_engine_numbers_and_keeps_any_other() {
    // `jtRigid`, `jtCloth`, `jtJoint`, `jtWheel`, `jtNone`, `jtSlider`.
    let named: [SkeletonJointKind; 6] = [
      SkeletonJointKind::Rigid,
      SkeletonJointKind::Cloth,
      SkeletonJointKind::Joint,
      SkeletonJointKind::Wheel,
      SkeletonJointKind::None,
      SkeletonJointKind::Slider,
    ];

    for (value, kind) in (0u32..).zip(named) {
      assert_eq!(SkeletonJointKind::from(value), kind);
      assert_eq!(u32::from(kind), value);
    }

    assert_eq!(SkeletonJointKind::from(6), SkeletonJointKind::Unknown(6));
    assert_eq!(u32::from(SkeletonJointKind::Unknown(u32::MAX)), u32::MAX);
  }
}
