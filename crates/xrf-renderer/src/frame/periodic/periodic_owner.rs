use crate::contract::render_viewport_id::RenderViewportId;

/// Who offers a periodic job, which a withdrawal takes back together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeriodicOwner {
  /// The scenes' own work, offered as they take in what the world posted, apart from any view.
  Scenes,
  /// A view's work, withdrawn when it draws nothing this frame.
  View(RenderViewportId),
}
