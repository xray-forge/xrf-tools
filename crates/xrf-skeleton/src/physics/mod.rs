//! A bone's physics as the formats carrying a skeleton store it: OGF visuals in `OGF_S_IKDATA`, editable objects in
//! their bone chunks.

mod skeleton_bone_shape;
mod skeleton_bone_shape_kind;
mod skeleton_joint_kind;
mod skeleton_joint_limit;

pub use skeleton_bone_shape::SkeletonBoneShape;
pub use skeleton_bone_shape_kind::SkeletonBoneShapeKind;
pub use skeleton_joint_kind::SkeletonJointKind;
pub use skeleton_joint_limit::SkeletonJointLimit;
