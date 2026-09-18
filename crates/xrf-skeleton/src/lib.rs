//! Skeleton motion data, shared by the formats that carry it: OGF visuals and OMF libraries.

pub(crate) mod skeleton_motion;
pub(crate) mod skeleton_motion_definition;
pub(crate) mod skeleton_motion_keys;
pub(crate) mod skeleton_motion_mark;
pub(crate) mod skeleton_motion_parameters_chunk;
pub(crate) mod skeleton_motions_chunk;
pub(crate) mod skeleton_part;

pub use crate::skeleton_motion::*;
pub use crate::skeleton_motion_definition::SkeletonMotionDefinition;
pub use crate::skeleton_motion_keys::{Quaternion, SAMPLE_FPS, SkeletonBoneMotion};
pub use crate::skeleton_motion_mark::SkeletonMotionMark;
pub use crate::skeleton_motion_parameters_chunk::SkeletonMotionParametersChunk;
pub use crate::skeleton_motions_chunk::SkeletonMotionsChunk;
pub use crate::skeleton_part::SkeletonPart;
