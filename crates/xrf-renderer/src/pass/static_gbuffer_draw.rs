/// How one G-buffer draw of the static scene goes.
#[derive(Clone, Copy, Debug, Default)]
pub struct StaticGbufferDraw {
  /// Whether it is the frame's first, which draws the impostors too: no occlusion sets one aside.
  pub is_first: bool,
  /// Whether the frame is drawn for an inspection view, through its own pipelines.
  pub is_inspected: bool,
  /// Whether the opaque surfaces' depth is drawn ahead of their shading.
  pub is_prepassed: bool,
}
