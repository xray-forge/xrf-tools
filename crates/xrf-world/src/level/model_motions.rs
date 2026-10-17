use std::sync::Arc;

use xrf_renderer::{RenderLevelSource, RenderMotion, RenderWorkers};

use crate::level::loader_slot::LoaderSlot;
use crate::level::loader_slots::LoaderSlots;

/// The motions a viewport's skinned models are posed by, each baked once on a loader thread the first time a pose
/// names it, and kept while the viewport shows the same scene.
pub struct ModelMotions {
  motions: LoaderSlots<RenderMotion>,
  workers: RenderWorkers,
}

impl ModelMotions {
  pub fn new(workers: &RenderWorkers) -> Self {
    Self {
      motions: LoaderSlots::default(),
      workers: workers.clone(),
    }
  }

  /// The motion by its name once it is baked, asking for it the first time; none while it is read or where it cannot
  /// be, which the caller poses the bind pose for.
  pub fn get(&mut self, source: &Arc<dyn RenderLevelSource>, name: &str) -> Option<&RenderMotion> {
    let workers: &RenderWorkers = &self.workers;

    self.motions.get(name, || {
      let (source, name) = (Arc::clone(source), name.to_owned());

      LoaderSlot::spawn(workers, move || {
        source
          .read_motion(&name)
          .inspect_err(|error| log::warn!("Motion '{name}' cannot be played: {error}"))
          .ok()
      })
    })
  }
}
