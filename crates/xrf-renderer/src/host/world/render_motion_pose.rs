use glam::{Mat4, Vec3};

/// Where an object motion has its object this frame, in engine space, and how fast it moved since the frame before
/// (`CCustomZone::OnMove`), which the lights and particles it carries follow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderMotionPose {
  pub transform: Mat4,
  /// Metres a second.
  pub velocity: Vec3,
}
