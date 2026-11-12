use crate::contract::view::output::render_debug_view::RenderDebugView;

/// Which of a detail slot's stored light values a debug view shows over the ground, as `shaders/grass/slot_light.wgsl`
/// numbers them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub(crate) enum GrassSlotValue {
  /// `c_dir`, the sun.
  Sun = 0,
  /// `c_hemi`, the hemisphere.
  Hemi = 1,
  /// `c_r`, `c_g` and `c_b`, the lamps' light, as one colour.
  Light = 2,
}

impl GrassSlotValue {
  /// The value a debug view shows, none for a view showing no slot.
  pub(crate) fn of(view: RenderDebugView) -> Option<Self> {
    match view {
      RenderDebugView::DetailSun => Some(Self::Sun),
      RenderDebugView::DetailHemi => Some(Self::Hemi),
      RenderDebugView::DetailLight => Some(Self::Light),
      _ => None,
    }
  }
}
