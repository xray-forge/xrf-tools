use crate::contract::world_viewport_event::WorldViewportEvent;

/// Where one viewport's world events go: in the application, the channel its page attached the viewport with.
pub trait WorldEventSink: Send + 'static {
  /// Sends an event.
  fn send(&self, event: WorldViewportEvent);
}
