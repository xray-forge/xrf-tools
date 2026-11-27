use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use xrf_renderer::{RenderEventSink, RenderFrameReport, RenderViewportEvent};

/// What the renderer tells the game of its viewport: a failure to draw goes to the log, whether the level stood ready
/// (every sector, particle and texture settled) to `ready`, whether a frame was drawn to `drawn`, and the latest frame
/// report to `frame`, for the developer layer's readout.
pub(crate) struct GameRenderSink {
  pub ready: Arc<AtomicBool>,
  pub drawn: Arc<AtomicBool>,
  pub frame: Arc<Mutex<Option<Box<RenderFrameReport>>>>,
}

impl RenderEventSink for GameRenderSink {
  fn send(&self, event: RenderViewportEvent) -> bool {
    match event {
      RenderViewportEvent::Failure { message } => log::error!("The renderer cannot draw the game's view: {message}"),
      RenderViewportEvent::Load { report } if report.is_ready => self.ready.store(true, Ordering::Release),
      RenderViewportEvent::Frame { report } => {
        self.drawn.store(true, Ordering::Release);
        *self.frame.lock().unwrap_or_else(PoisonError::into_inner) = Some(report);
      }
      _ => {}
    }

    true
  }
}
