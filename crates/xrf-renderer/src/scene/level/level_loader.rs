use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel};

use xrf_visual::SectorPackage;

use crate::host::render_level_source::RenderLevelSource;
use crate::scene::level::surface_tally::SurfaceTally;
use crate::thread::render_workers::RenderWorkers;

/// One sector's pack as a loader thread finished it, with what its surfaces draw, or why it could not be packed.
pub type SectorLoad = (u32, Result<(SectorPackage, SurfaceTally), String>);

/// Packs every sector of a level side by side on the loader threads, handing each to the render thread as it is done.
/// Dropped, it lets go of the sectors not started yet, so a level closed mid open stops costing work.
pub struct LevelLoader {
  receiver: Receiver<SectorLoad>,
  total: u32,
  received: u32,
  is_cancelled: Arc<AtomicBool>,
}

impl LevelLoader {
  pub fn start(source: Arc<dyn RenderLevelSource>, workers: &RenderWorkers) -> Self {
    let (sender, receiver) = channel();
    let total: u32 = source.get_sector_count();
    let is_cancelled: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    for sector in 0..total {
      let (sender, source, is_cancelled) = (sender.clone(), Arc::clone(&source), Arc::clone(&is_cancelled));

      workers.spawn(move || {
        if is_cancelled.load(Ordering::Acquire) {
          return;
        }

        let pack: Result<(SectorPackage, SurfaceTally), String> = source
          .pack_sector(sector)
          .map(|package| {
            let tally: SurfaceTally = SurfaceTally::measure(&package);

            (package, tally)
          })
          .map_err(|error| error.to_string());

        if let Err(error) = &pack {
          log::error!("Sector {sector} cannot be drawn: {error}");
        }

        let _ = sender.send((sector, pack));
      });
    }

    Self {
      receiver,
      total,
      received: 0,
      is_cancelled,
    }
  }

  pub fn get_total(&self) -> u32 {
    self.total
  }

  /// The sectors finished since the last call, at most `limit` of them.
  pub fn take(&mut self, limit: usize) -> Vec<SectorLoad> {
    let taken: Vec<SectorLoad> = self.receiver.try_iter().take(limit).collect();

    self.received += taken.len() as u32;

    taken
  }
}

impl Drop for LevelLoader {
  fn drop(&mut self) {
    self.is_cancelled.store(true, Ordering::Release);
  }
}
