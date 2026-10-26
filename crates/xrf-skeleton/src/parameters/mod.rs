//! The motion parameters chunk, `OGF_S_SMPARAMS`: the bone partitions and one definition per motion, with its flags
//! and marks.

mod skeleton_motion_definition;
mod skeleton_motion_flag;
mod skeleton_motion_mark;
mod skeleton_motion_parameters_chunk;
mod skeleton_part;

pub use skeleton_motion_definition::SkeletonMotionDefinition;
pub use skeleton_motion_flag::SkeletonMotionFlag;
pub use skeleton_motion_mark::SkeletonMotionMark;
pub use skeleton_motion_parameters_chunk::SkeletonMotionParametersChunk;
pub use skeleton_part::SkeletonPart;
