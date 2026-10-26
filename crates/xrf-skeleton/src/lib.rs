#![doc = include_str!("../README.md")]

mod motions;
mod parameters;

pub use crate::motions::{Quaternion, SkeletonBoneMotion, SkeletonMotion, SkeletonMotionsChunk};
pub use crate::parameters::{
  SkeletonMotionDefinition, SkeletonMotionFlag, SkeletonMotionMark, SkeletonMotionParametersChunk, SkeletonPart,
};
