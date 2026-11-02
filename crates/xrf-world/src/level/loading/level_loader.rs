use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;

use xrf_renderer::{LoaderReceiver, RenderWorkers};
use xrf_visual::SectorPackage;

use crate::host::world_level_source::WorldLevelSource;
use crate::level::loading::loader_flight::LoaderFlight;
use crate::level::loading::sector_load::SectorLoad;
use crate::level::streaming::surface_tally::SurfaceTally;

/// Packs a level's sectors side by side on the loader threads, no more than the pool's width of them running or queued
/// at once (`LoaderFlight`), handing each to the render thread as it is done; the next start as the render thread takes
/// the finished ones, once a frame. Dropped, it lets go of the sectors not started yet, so a level closed mid open
/// stops costing work.
pub struct LevelLoader {
  source: Arc<dyn WorldLevelSource>,
  workers: RenderWorkers,
  sender: Sender<SectorLoad>,
  receiver: LoaderReceiver<SectorLoad>,
  total: u32,
  /// The sectors started, in order, and how many of them finished.
  flight: LoaderFlight,
  is_cancelled: Arc<AtomicBool>,
}

impl LevelLoader {
  pub fn start(source: Arc<dyn WorldLevelSource>, workers: &RenderWorkers) -> Self {
    let (sender, receiver) = LoaderReceiver::channel();
    let mut loader: Self = Self {
      total: source.get_sector_count(),
      source,
      workers: workers.clone(),
      sender,
      receiver,
      flight: LoaderFlight::new(workers),
      is_cancelled: Arc::new(AtomicBool::new(false)),
    };

    loader.refill();

    loader
  }

  pub fn get_total(&self) -> u32 {
    self.total
  }

  /// The sectors finished since the last call, at most `limit` of them; starts as many more as finished since the last
  /// call, whether or not they are taken yet.
  pub fn take(&mut self, limit: usize) -> Vec<SectorLoad> {
    let taken: Vec<SectorLoad> = self.receiver.try_iter().take(limit).collect();

    self.refill();

    taken
  }

  /// Starts the next sectors while the pool's width of them are not packing.
  fn refill(&mut self) {
    while self.flight.get_started() < self.total
      && let Some(finished) = self.flight.start()
    {
      let sector: u32 = self.flight.get_started() - 1;
      let (sender, source, is_cancelled) = (
        self.sender.clone(),
        Arc::clone(&self.source),
        Arc::clone(&self.is_cancelled),
      );

      self.workers.spawn(move || {
        if !is_cancelled.load(Ordering::Acquire) {
          let _ = sender.send(SectorLoad {
            sector,
            pack: Self::pack(source.as_ref(), sector),
          });
        }

        finished.fetch_add(1, Ordering::AcqRel);
      });
    }
  }

  /// One sector packed, with what its surfaces draw.
  fn pack(source: &dyn WorldLevelSource, sector: u32) -> Result<(SectorPackage, SurfaceTally), String> {
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

    pack
  }
}

impl Drop for LevelLoader {
  fn drop(&mut self) {
    self.is_cancelled.store(true, Ordering::Release);
  }
}
