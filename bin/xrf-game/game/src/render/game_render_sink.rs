use xrf_renderer::{RenderEventSink, RenderViewportEvent};

/// What the renderer tells the game of its viewport: a failure to draw goes to the log, the frame and load reports are
/// for the developer layer and the control endpoint once they come.
pub(crate) struct GameRenderSink;

impl RenderEventSink for GameRenderSink {
  fn send(&self, event: RenderViewportEvent) -> bool {
    if let RenderViewportEvent::Failure { message } = event {
      log::error!("The renderer cannot draw the game's view: {message}");
    }

    true
  }
}
