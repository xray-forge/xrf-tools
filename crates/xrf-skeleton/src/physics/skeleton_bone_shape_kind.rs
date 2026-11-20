/// Which of its three shapes a bone collides as, `SBoneShape::EShapeType`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkeletonBoneShapeKind {
  /// The bone does not collide.
  #[default]
  None,
  Box,
  Sphere,
  Cylinder,
  /// A value the engine does not name, kept so it writes back as stored.
  Unknown(u16),
}

impl From<u16> for SkeletonBoneShapeKind {
  fn from(value: u16) -> Self {
    match value {
      0 => Self::None,
      1 => Self::Box,
      2 => Self::Sphere,
      3 => Self::Cylinder,
      other => Self::Unknown(other),
    }
  }
}

impl From<SkeletonBoneShapeKind> for u16 {
  fn from(kind: SkeletonBoneShapeKind) -> Self {
    match kind {
      SkeletonBoneShapeKind::None => 0,
      SkeletonBoneShapeKind::Box => 1,
      SkeletonBoneShapeKind::Sphere => 2,
      SkeletonBoneShapeKind::Cylinder => 3,
      SkeletonBoneShapeKind::Unknown(value) => value,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::SkeletonBoneShapeKind;

  #[test]
  fn names_the_shapes_the_engine_numbers_and_keeps_any_other() {
    // `stNone`, `stBox`, `stSphere`, `stCylinder`.
    let named: [SkeletonBoneShapeKind; 4] = [
      SkeletonBoneShapeKind::None,
      SkeletonBoneShapeKind::Box,
      SkeletonBoneShapeKind::Sphere,
      SkeletonBoneShapeKind::Cylinder,
    ];

    for (value, kind) in (0u16..).zip(named) {
      assert_eq!(SkeletonBoneShapeKind::from(value), kind);
      assert_eq!(u16::from(kind), value);
    }

    assert_eq!(SkeletonBoneShapeKind::from(7), SkeletonBoneShapeKind::Unknown(7));
    assert_eq!(u16::from(SkeletonBoneShapeKind::Unknown(0xffff)), 0xffff);
  }
}
