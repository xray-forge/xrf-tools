use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use xrf_renderer::{
  RenderBundle, RenderEventSink, RenderLevelSource, RenderViewportId, RenderWindowHost, RenderWorkers, Renderer,
};

use crate::plugins::render::viewport_windows::ViewportWindows;

/// The application's one renderer, which starts a GPU only once a viewport is attached.
pub struct RenderState {
  pub renderer: Renderer,
  /// The latest show asked of each viewport, so one that took longer to prepare never replaces one asked after it.
  shows: Mutex<HashMap<RenderViewportId, u64>>,
  /// The window each viewport draws into, so a page that goes takes its viewports with it.
  windows: Mutex<ViewportWindows>,
}

impl RenderState {
  /// A renderer whose loaders read and decode on `workers`, reading the files it ships with from `bundle`.
  pub fn new(workers: RenderWorkers, bundle: Arc<dyn RenderBundle>) -> Self {
    Self {
      renderer: Renderer::new(workers, bundle),
      shows: Mutex::default(),
      windows: Mutex::default(),
    }
  }

  /// Starts drawing a viewport into a window, named by its label, telling its page through `sink`.
  pub fn attach_viewport(
    &self,
    window: &str,
    host: Arc<dyn RenderWindowHost>,
    sink: Box<dyn RenderEventSink>,
  ) -> RenderViewportId {
    let viewport: RenderViewportId = self.renderer.attach_viewport(host, sink);

    self.lock_windows().insert(viewport, window);

    log::info!("Attached native viewport {} to window '{window}'", viewport.0);

    viewport
  }

  /// Stops drawing a viewport its page let go.
  pub fn detach_viewport(&self, viewport: RenderViewportId) {
    let window: Option<String> = self.lock_windows().remove(viewport);

    match window {
      Some(window) => log::info!("Detached native viewport {} from window '{window}'", viewport.0),
      None => log::info!("Detached native viewport {}, which no window held", viewport.0),
    }

    self.release(viewport);
  }

  /// Stops drawing every viewport in a window whose page is loading again, which can no longer let them go itself.
  pub fn detach_window(&self, window: &str) {
    let viewports: Vec<RenderViewportId> = self.lock_windows().take_window(window);

    for viewport in viewports {
      log::info!(
        "Detached native viewport {} from window '{window}', whose page is loading again",
        viewport.0
      );

      self.release(viewport);
    }
  }

  /// A ticket for what a viewport is asked to show next, which every earlier ticket of it gives way to.
  pub fn ask_show(&self, viewport: RenderViewportId) -> u64 {
    let mut shows: MutexGuard<'_, HashMap<RenderViewportId, u64>> = self.lock_shows();
    let ticket: &mut u64 = shows.entry(viewport).or_default();

    *ticket += 1;
    *ticket
  }

  /// Shows a source in a viewport, none for `None`, unless something was asked of it after `ticket`.
  pub fn show(&self, viewport: RenderViewportId, ticket: u64, source: Option<Arc<dyn RenderLevelSource>>) {
    let shows: MutexGuard<'_, HashMap<RenderViewportId, u64>> = self.lock_shows();

    if shows.get(&viewport) == Some(&ticket) {
      self.renderer.show_level(viewport, source);
    } else {
      log::info!("Native viewport {} was asked to show something newer", viewport.0);
    }
  }

  fn release(&self, viewport: RenderViewportId) {
    self.renderer.detach_viewport(viewport);
    self.lock_shows().remove(&viewport);
  }

  fn lock_shows(&self) -> MutexGuard<'_, HashMap<RenderViewportId, u64>> {
    self.shows.lock().unwrap_or_else(PoisonError::into_inner)
  }

  fn lock_windows(&self) -> MutexGuard<'_, ViewportWindows> {
    self.windows.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
