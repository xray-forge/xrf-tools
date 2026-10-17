//! The sectors a level packs on the workers: a pool's width at a time, the next started when the finished are taken.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use xrf_renderer::{RenderLevelSource, RenderWorkers};

use crate::level::level_loader::{LevelLoader, SectorLoad};
use crate::tests::fixtures::TestLevelSource;
use crate::tests::test_workers::create_workers;

/// Longer than any of these waits should take, so a broken bound fails rather than hangs.
const WAIT: Duration = Duration::from_secs(10);

/// A level of `sectors` sectors, each packing into nothing once `gate` lets it (at once where none is given), counting
/// how many were packed.
fn counted_source(sectors: u32, gate: Option<Receiver<()>>) -> (Arc<dyn RenderLevelSource>, Arc<AtomicU32>) {
  let packed: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
  let counter: Arc<AtomicU32> = Arc::clone(&packed);
  let gate: Mutex<Option<Receiver<()>>> = Mutex::new(gate);
  let source: Arc<dyn RenderLevelSource> = Arc::new(TestLevelSource {
    sectors,
    pack: Box::new(move |_| {
      if let Some(gate) = gate.lock().unwrap_or_else(PoisonError::into_inner).as_ref() {
        let _ = gate.recv_timeout(WAIT);
      }

      counter.fetch_add(1, Ordering::AcqRel);

      Err(TestLevelSource::none())
    }),
    ..TestLevelSource::default()
  });

  (source, packed)
}

// Work queued on the pool after the loader started runs once the first round of sectors is packed, not after the
// level: no more of them than the pool is wide wait ahead of it.
#[test]
fn queues_no_more_sectors_than_the_pool_is_wide_ahead_of_other_work() {
  let workers: RenderWorkers = create_workers();
  let (open, gate): (Sender<()>, Receiver<()>) = channel();
  let (source, packed) = counted_source(10, Some(gate));
  let _loader: LevelLoader = LevelLoader::start(source, &workers);
  let (seen, packed_before): (Sender<u32>, Receiver<u32>) = channel();
  let counter: Arc<AtomicU32> = Arc::clone(&packed);

  workers.spawn(move || {
    let _ = seen.send(counter.load(Ordering::Acquire));
  });
  drop(open);

  let before: u32 = packed_before
    .recv_timeout(WAIT)
    .expect("the work queued after the loader runs");

  assert!(before <= workers.get_width() as u32, "{before} sectors packed first");
}

#[test]
fn packs_every_sector_once_as_they_are_taken() {
  let (source, packed) = counted_source(10, None);
  let mut loader: LevelLoader = LevelLoader::start(source, &create_workers());
  let mut taken: Vec<SectorLoad> = Vec::new();
  let started: Instant = Instant::now();

  while taken.len() < 10 && started.elapsed() < WAIT {
    taken.extend(loader.take(4));
    std::thread::sleep(Duration::from_millis(1));
  }

  let mut sectors: Vec<u32> = taken.iter().map(|(sector, _)| *sector).collect();

  sectors.sort_unstable();

  assert_eq!(sectors, (0..10).collect::<Vec<u32>>());
  assert_eq!(packed.load(Ordering::Acquire), 10);
}
