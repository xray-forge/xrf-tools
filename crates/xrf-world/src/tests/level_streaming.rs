//! How a level streams into its scene: what failed is counted once, and a level let go mid-load leaves nothing behind.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_renderer::{RenderSceneFrame, RenderSceneId, RenderViewportId, RenderWorld};

use crate::tests::fixtures::TestLevelSource;
use crate::tests::test_workers::create_workers;
use crate::world::World;

/// Longer than streaming a level of a few sectors on the test workers should take.
const WAIT: Duration = Duration::from_secs(10);

/// Advances the world's scenes until one's frame meets `until`, answering that frame.
fn advance_until(world: &mut World, until: impl Fn(&RenderSceneFrame) -> bool) -> Option<RenderSceneFrame> {
  let started: Instant = Instant::now();

  while started.elapsed() < WAIT {
    let frames: HashMap<RenderSceneId, RenderSceneFrame> = world.advance_scenes(Instant::now(), HashMap::new());

    if let Some(frame) = frames.into_values().find(|frame| until(frame)) {
      return Some(frame);
    }

    std::thread::sleep(Duration::from_millis(2));
  }

  None
}

#[test]
fn finishes_streaming_a_level_whose_sectors_all_fail_counting_each_once() {
  let mut world: World = World::new(create_workers());
  let viewport: RenderViewportId = RenderViewportId(1);

  // The fixture packs no sector: every one of the three fails.
  world.show_level(
    viewport,
    Some(Arc::new(TestLevelSource {
      sectors: 3,
      ..TestLevelSource::default()
    })),
  );

  let done: RenderSceneFrame =
    advance_until(&mut world, |frame| frame.streaming.is_sectors_done).expect("the sectors done");

  assert_eq!((done.streaming.sectors, done.streaming.sectors_total), (0, 3));

  for _ in 0..8 {
    world.advance_scenes(Instant::now(), HashMap::new());
  }

  assert_eq!(world.describe_problems(viewport).sectors.len(), 3);
}

#[test]
fn leaves_no_scene_behind_a_level_let_go_while_it_streams() {
  let mut world: World = World::new(create_workers());
  let viewport: RenderViewportId = RenderViewportId(1);

  world.show_level(
    viewport,
    Some(Arc::new(TestLevelSource {
      sectors: 64,
      ..TestLevelSource::default()
    })),
  );
  world.advance_scenes(Instant::now(), HashMap::new());
  world.show_level(viewport, None);

  for _ in 0..8 {
    assert!(world.advance_scenes(Instant::now(), HashMap::new()).is_empty());
    std::thread::sleep(Duration::from_millis(2));
  }
}
