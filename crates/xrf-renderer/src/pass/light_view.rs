use crate::contract::render_debug_view::RenderDebugView;

/// What the sun and the lamps draw into the light target, as `shaders/common/camera.wgsl` numbers it: their light, or
/// what a debug view shows of them in its place.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum LightView {
  /// Their light.
  Light = 0,
  /// The sun the cascade it reads, from one, and how much of it reaches; the lamps nothing.
  Cascades = 1,
  /// The lamps the count their cluster lists; the sun nothing.
  LampsPerCluster = 2,
  /// Neither anything, which the overdraw's count is added into.
  Overdraw = 3,
}

impl LightView {
  /// What the sun and the lamps draw for a debug view.
  pub fn of(view: RenderDebugView) -> Self {
    match view {
      RenderDebugView::Cascades => Self::Cascades,
      RenderDebugView::LampsPerCluster => Self::LampsPerCluster,
      RenderDebugView::Overdraw => Self::Overdraw,
      _ => Self::Light,
    }
  }
}
