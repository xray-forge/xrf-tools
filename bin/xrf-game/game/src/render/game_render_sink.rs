use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use xrf_renderer::{RenderEventSink, RenderFrameReport, RenderLoadReport, RenderViewportEvent};

/// What the renderer tells the game of its viewport: a failure to draw goes to the log, how far the level loaded to
/// `load` and whether it stood ready (every sector, particle and texture settled) to `ready`, how many frames were
/// drawn to `drawn`, and the latest frame report to `frame`, for the developer layer's readout.
pub(crate) struct GameRenderSink {
  pub ready: Arc<AtomicBool>,
  pub load: Arc<Mutex<Option<RenderLoadReport>>>,
  pub drawn: Arc<AtomicU64>,
  pub frame: Arc<Mutex<Option<Box<RenderFrameReport>>>>,
}

impl RenderEventSink for GameRenderSink {
  fn send(&self, event: RenderViewportEvent) -> bool {
    match event {
      RenderViewportEvent::Failure { message } => log::error!("The renderer cannot draw the game's view: {message}"),
      RenderViewportEvent::Load { report } => {
        if report.is_ready {
          self.ready.store(true, Ordering::Release);
        }

        *self.load.lock().unwrap_or_else(PoisonError::into_inner) = Some(report);
      }
      RenderViewportEvent::Frame { report } => {
        self.drawn.fetch_add(1, Ordering::AcqRel);
        *self.frame.lock().unwrap_or_else(PoisonError::into_inner) = Some(report);
      }
      RenderViewportEvent::Applied { .. } => {}
    }

    true
  }
}
