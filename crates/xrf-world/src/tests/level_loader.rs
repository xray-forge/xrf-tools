//! The sectors a level packs on the workers: a pool's width at a time, the next started as they are taken.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use xrf_renderer::{RenderLevelSource, RenderWorkers};

use crate::level::level_loader::{LevelLoader, SectorLoad};
use crate::tests::fixtures::TestLevelSource;
use crate::tests::test_workers::create_workers;

/// A level of `sectors` sectors, each packing at once into nothing, counting how many were packed.
fn counted_source(sectors: u32) -> (Arc<dyn RenderLevelSource>, Arc<AtomicU32>) {
  let packed: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
  let counter: Arc<AtomicU32> = Arc::clone(&packed);
  let source: Arc<dyn RenderLevelSource> = Arc::new(TestLevelSource {
    sectors,
    pack: Box::new(move |_| {
      counter.fetch_add(1, Ordering::AcqRel);

      Err(TestLevelSource::none())
    }),
    ..TestLevelSource::default()
  });

  (source, packed)
}

#[test]
fn packs_no_more_sectors_than_the_pool_is_wide_until_they_are_taken() {
  let workers: RenderWorkers = create_workers();
  let (source, packed) = counted_source(10);
  let _loader: LevelLoader = LevelLoader::start(source, &workers);

  std::thread::sleep(Duration::from_millis(200));

  assert_eq!(packed.load(Ordering::Acquire), workers.get_width() as u32);
}

#[test]
fn packs_every_sector_once_as_they_are_taken() {
  let (source, packed) = counted_source(10);
  let mut loader: LevelLoader = LevelLoader::start(source, &create_workers());
  let mut taken: Vec<SectorLoad> = Vec::new();
  let started: Instant = Instant::now();

  while taken.len() < 10 && started.elapsed() < Duration::from_secs(5) {
    taken.extend(loader.take(4));
    std::thread::sleep(Duration::from_millis(1));
  }

  let mut sectors: Vec<u32> = taken.iter().map(|(sector, _)| *sector).collect();

  sectors.sort_unstable();

  assert_eq!(sectors, (0..10).collect::<Vec<u32>>());
  assert_eq!(packed.load(Ordering::Acquire), 10);
}
