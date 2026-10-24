use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use xrf_renderer::{RenderLevelSource, RenderSceneId, RenderViewportId};

use crate::WorldState;

/// What each viewport was asked to show last, so a show that took longer to prepare never replaces one asked after it,
/// and none reaches a viewport detached since it was asked. A clone orders the same shows.
#[derive(Clone)]
pub struct ViewportShows {
  tickets: Arc<Mutex<HashMap<RenderViewportId, u64>>>,
  world: WorldState,
}

impl ViewportShows {
  /// The shows of the viewports drawing `world`.
  pub fn new(world: WorldState) -> Self {
    Self {
      tickets: Arc::default(),
      world,
    }
  }

  /// A ticket for what a viewport is asked to show next, which every earlier ticket of it gives way to.
  pub fn ask(&self, viewport: RenderViewportId) -> u64 {
    let mut tickets: MutexGuard<'_, HashMap<RenderViewportId, u64>> = self.lock();
    let ticket: &mut u64 = tickets.entry(viewport).or_default();

    *ticket += 1;
    *ticket
  }

  /// Shows a source in a viewport, none for `None`, unless something was asked of it after `ticket` or it was detached
  /// since; answers the scene shown.
  pub fn show(
    &self,
    viewport: RenderViewportId,
    ticket: u64,
    source: Option<Arc<dyn RenderLevelSource>>,
  ) -> Option<RenderSceneId> {
    let tickets: MutexGuard<'_, HashMap<RenderViewportId, u64>> = self.lock();

    if tickets.get(&viewport) == Some(&ticket) {
      self.world.lock().show_level(viewport, source)
    } else {
      log::info!("Native viewport {} was asked to show something newer", viewport.0);

      None
    }
  }

  /// Forgets a detached viewport, so no show asked before its detach reaches the world, which would make it again.
  pub(crate) fn forget(&self, viewport: RenderViewportId) {
    self.lock().remove(&viewport);
  }

  fn lock(&self) -> MutexGuard<'_, HashMap<RenderViewportId, u64>> {
    self.tickets.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
