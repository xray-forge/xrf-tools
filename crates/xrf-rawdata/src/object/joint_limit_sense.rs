use xrf_skeleton::SkeletonJointLimit;

/// The editor and the physics engine turn a joint in opposite senses, so the OGF stores each limit flipped.
pub(crate) struct JointLimitSense;

impl JointLimitSense {
  /// The limit in the other sense: `SJointIKData::Export` stores the authored `(min, max)` as `(-max, -min)`, and the
  /// same flip turns a stored limit back into the authored one.
  pub(crate) fn flip(limit: SkeletonJointLimit) -> SkeletonJointLimit {
    SkeletonJointLimit {
      min: -limit.max,
      max: -limit.min,
      ..limit
    }
  }
}
