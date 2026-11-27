use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::task::Waker;

use xrf_renderer::{RenderEventSink, RenderFrameReport, RenderLoadReport, RenderViewportEvent};

/// What the renderer tells the game of its viewport: a failure to draw goes to the log, how far the level loaded to
/// `load` and whether it stood ready (every sector, particle and texture settled) to `ready`, how many frames were
/// drawn to `drawn`, and the latest frame report to `frame`, for the developer layer's readout. The main thread is
/// woken as the level first stands ready, which ends its load; the rest it reads when it next has work.
pub(crate) struct GameRenderSink {
  pub ready: Arc<AtomicBool>,
  pub load: Arc<Mutex<Option<RenderLoadReport>>>,
  pub drawn: Arc<AtomicU64>,
  pub frame: Arc<Mutex<Option<Box<RenderFrameReport>>>>,
  pub waker: Waker,
}

impl RenderEventSink for GameRenderSink {
  fn send(&self, event: RenderViewportEvent) -> bool {
    match event {
      RenderViewportEvent::Failure { message } => log::error!("The renderer cannot draw the game's view: {message}"),
      RenderViewportEvent::Load { report } => {
        let is_first_ready: bool = report.is_ready && !self.ready.swap(true, Ordering::AcqRel);

        *self.load.lock().unwrap_or_else(PoisonError::into_inner) = Some(report);

        if is_first_ready {
          self.waker.wake_by_ref();
        }
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
