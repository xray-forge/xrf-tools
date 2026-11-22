use xrf_math::glam::{Mat4, Vec3};

/// An object motion's pose this frame in engine space and its velocity since the last (`CCustomZone::OnMove`), which
/// the lights and particles it carries follow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderMotionPose {
  pub transform: Mat4,
  /// Metres per second.
  pub velocity: Vec3,
}
