#![doc = include_str!("../README.md")]

pub(crate) mod quaternion;
pub(crate) mod skeleton_bone_motion;
pub(crate) mod skeleton_motion;
pub(crate) mod skeleton_motion_definition;
pub(crate) mod skeleton_motion_flag;
pub(crate) mod skeleton_motion_keys;
pub(crate) mod skeleton_motion_mark;
pub(crate) mod skeleton_motion_parameters_chunk;
pub(crate) mod skeleton_motions_chunk;
pub(crate) mod skeleton_part;

pub use crate::quaternion::Quaternion;
pub use crate::skeleton_bone_motion::SkeletonBoneMotion;
pub use crate::skeleton_motion::*;
pub use crate::skeleton_motion_definition::SkeletonMotionDefinition;
pub use crate::skeleton_motion_flag::SkeletonMotionFlag;
pub use crate::skeleton_motion_keys::SAMPLE_FPS;
pub use crate::skeleton_motion_mark::SkeletonMotionMark;
pub use crate::skeleton_motion_parameters_chunk::SkeletonMotionParametersChunk;
pub use crate::skeleton_motions_chunk::SkeletonMotionsChunk;
pub use crate::skeleton_part::SkeletonPart;
