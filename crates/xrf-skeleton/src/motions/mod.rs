//! The motions chunk, `OGF_S_MOTIONS`: a count, then each motion's key runs, and what decoding them yields per bone.

mod quaternion;
mod skeleton_bone_motion;
mod skeleton_motion;
mod skeleton_motion_cursor;
mod skeleton_motions_chunk;

pub use quaternion::Quaternion;
pub use skeleton_bone_motion::SkeletonBoneMotion;
pub use skeleton_motion::SkeletonMotion;
pub(crate) use skeleton_motion_cursor::SkeletonMotionCursor;
pub use skeleton_motions_chunk::SkeletonMotionsChunk;
