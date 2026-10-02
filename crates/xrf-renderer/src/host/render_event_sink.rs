use crate::contract::render_viewport_event::RenderViewportEvent;

/// Where one viewport's events go: in the application, the channel its page attached the viewport with.
pub trait RenderEventSink: Send + 'static {
  /// Sends an event, answering whether anyone could still hear it.
  fn send(&self, event: RenderViewportEvent) -> bool;
}
