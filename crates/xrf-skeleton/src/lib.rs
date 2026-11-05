#![doc = include_str!("../README.md")]

mod motions;
mod parameters;
mod physics;

pub use crate::motions::{Quaternion, SkeletonBoneMotion, SkeletonMotion, SkeletonMotionsChunk};
pub use crate::parameters::{
  SkeletonMotionDefinition, SkeletonMotionFlag, SkeletonMotionMark, SkeletonMotionParametersChunk, SkeletonPart,
};
pub use crate::physics::{SkeletonBoneShape, SkeletonBoneShapeKind, SkeletonJointKind, SkeletonJointLimit};
