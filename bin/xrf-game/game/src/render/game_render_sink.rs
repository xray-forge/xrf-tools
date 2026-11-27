use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use xrf_renderer::{RenderEventSink, RenderViewportEvent};

/// What the renderer tells the game of its viewport: a failure to draw goes to the log, and whether the level stood
/// ready (every sector, particle and texture settled) to `ready`. The frame reports wait for the developer layer.
pub(crate) struct GameRenderSink {
  pub ready: Arc<AtomicBool>,
}

impl RenderEventSink for GameRenderSink {
  fn send(&self, event: RenderViewportEvent) -> bool {
    match event {
      RenderViewportEvent::Failure { message } => log::error!("The renderer cannot draw the game's view: {message}"),
      RenderViewportEvent::Load { report } if report.is_ready => self.ready.store(true, Ordering::Release),
      _ => {}
    }

    true
  }
}
