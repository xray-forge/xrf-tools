use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use xrf_renderer::{RenderLevelSource, RenderViewportId, Renderer};

/// The application's one renderer, which starts a GPU only once a viewport is attached.
#[derive(Default)]
pub struct RenderState {
  pub renderer: Renderer,
  /// The latest show asked of each viewport, so one that took longer to prepare never replaces one asked after it.
  shows: Mutex<HashMap<RenderViewportId, u64>>,
}

impl RenderState {
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

  /// Forgets a detached viewport's tickets.
  pub fn forget(&self, viewport: RenderViewportId) {
    self.lock_shows().remove(&viewport);
  }

  fn lock_shows(&self) -> MutexGuard<'_, HashMap<RenderViewportId, u64>> {
    self.shows.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
