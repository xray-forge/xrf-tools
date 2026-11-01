use glam::Mat4;

/// What one frame's occlusion search runs over: the camera's projection, the search target's size, the noise's frame,
/// and the histories it carries.
#[derive(Clone, Copy, Debug)]
pub struct VbaoFrame {
  pub projection: Mat4,
  /// The search target's width and height.
  pub size: (u32, u32),
  pub frame: u32,
  /// Whether the occlusion's history holds a frame to carry.
  pub has_history: bool,
  /// Whether the indirect light's history holds one.
  pub has_light_history: bool,
}
