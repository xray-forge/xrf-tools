use std::sync::Arc;

use xrf_renderer::{RenderMotion, RenderWorkers};

use crate::host::world_level_source::WorldLevelSource;
use crate::level::loading::loader_slot::LoaderSlot;
use crate::level::loading::loader_slots::LoaderSlots;

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
  pub fn get(&mut self, source: &Arc<dyn WorldLevelSource>, name: &str) -> Option<&RenderMotion> {
    let workers: &RenderWorkers = &self.workers;

    self.motions.get(name, || {
      let (source, name) = (Arc::clone(source), name.to_owned());

      LoaderSlot::spawn(workers, "motion", move || {
        source
          .read_motion(&name)
          .inspect_err(|error| log::warn!("Motion '{name}' cannot be played: {error}"))
          .ok()
      })
    })
  }
}
