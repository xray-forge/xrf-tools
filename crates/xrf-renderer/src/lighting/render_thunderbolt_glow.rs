use glam::Vec3;

/// One of a striking bolt's glows where it stands this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderThunderboltGlow {
  /// In renderer space.
  pub position: Vec3,
  /// Half its width and half its height, in metres.
  pub extent: [f32; 2],
  /// What its colour and alpha are both scaled by.
  pub opacity: f32,
}
