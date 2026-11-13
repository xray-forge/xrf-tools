use crate::contract::view::output::render_debug_view::RenderDebugView;

/// Which of an AI map node's values a debug view shows over the ground, as `shaders/ai_map/node_field.wgsl` numbers
/// them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub(crate) enum AiMapField {
  /// The mean of its four covers from standing height.
  HighCover = 0,
  /// The mean of its four covers from crouching height.
  LowCover = 1,
  /// How many of its four neighbours it lacks.
  Links = 2,
}

impl AiMapField {
  /// The field a debug view shows, none for a view showing no node.
  pub(crate) fn of(view: RenderDebugView) -> Option<Self> {
    match view {
      RenderDebugView::AiHighCover => Some(Self::HighCover),
      RenderDebugView::AiLowCover => Some(Self::LowCover),
      RenderDebugView::AiLinks => Some(Self::Links),
      _ => None,
    }
  }
}
