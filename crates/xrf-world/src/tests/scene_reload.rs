//! A scene moved onto a next source of its level: it keeps its id and viewers, and reads again only what it is told.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_renderer::{RenderLevelSource, RenderSceneFrame, RenderSceneId, RenderViewportId, RenderWorld};

use crate::contract::world_level_reload::WorldLevelReload;
use crate::tests::fixtures::{SilentSink, TestLevelReads, TestLevelSource};
use crate::tests::test_workers::create_workers;
use crate::world::World;

/// Longer than a test level's reads take on the test workers.
const WAIT: Duration = Duration::from_secs(10);

/// What a reload is given time to start reading before a test says it read nothing.
const QUIET: Duration = Duration::from_millis(100);

/// A source of the level `zaton` counting its reads into `reads`.
fn source(reads: &Arc<TestLevelReads>) -> Arc<dyn RenderLevelSource> {
  Arc::new(TestLevelSource {
    key: Some(String::from("zaton")),
    reads: Arc::clone(reads),
    ..TestLevelSource::default()
  })
}

/// Advances the world's scenes until `reads` has counted `expected`, answering the last frames.
fn advance_until_read(
  world: &mut World,
  reads: &TestLevelReads,
  expected: [u32; 4],
) -> HashMap<RenderSceneId, RenderSceneFrame> {
  let started: Instant = Instant::now();

  loop {
    let frames: HashMap<RenderSceneId, RenderSceneFrame> = world.advance_scenes(Instant::now(), HashMap::new());

    if reads.count() == expected || started.elapsed() > WAIT {
      return frames;
    }

    std::thread::sleep(Duration::from_millis(2));
  }
}

/// A world with one viewport showing the level, its first reads done: lights, weather, spawn and particles once each.
fn show(reads: &Arc<TestLevelReads>) -> (World, RenderViewportId, RenderSceneId) {
  let mut world: World = World::new(create_workers());
  let viewport: RenderViewportId = RenderViewportId(1);

  world.attach(viewport, Box::new(SilentSink));

  let scene: RenderSceneId = world
    .show_level(viewport, Some(source(reads)))
    .expect("the level shown");

  advance_until_read(&mut world, reads, [1, 1, 1, 1]);
  assert_eq!(reads.count(), [1, 1, 1, 1]);

  (world, viewport, scene)
}

#[test]
fn a_next_source_keeps_the_scene_its_viewers_and_everything_drawn() {
  let reads: Arc<TestLevelReads> = Arc::default();
  let (mut world, viewport, scene) = show(&reads);
  let next: Arc<dyn RenderLevelSource> = source(&reads);

  assert_eq!(
    world.reload_scene("zaton", Arc::clone(&next), WorldLevelReload::default()),
    Some(scene)
  );
  assert_eq!(world.get_asked_scene(viewport), Some(scene));

  std::thread::sleep(QUIET);

  let frames: HashMap<RenderSceneId, RenderSceneFrame> = world.advance_scenes(Instant::now(), HashMap::new());

  assert_eq!(frames.keys().copied().collect::<Vec<_>>(), vec![scene]);
  assert!(Arc::ptr_eq(&frames[&scene].source, &next));
  assert!(frames[&scene].updates.is_empty());
  // An unaffected commit reads nothing again.
  assert_eq!(reads.count(), [1, 1, 1, 1]);
}

#[test]
fn a_reload_reads_again_only_what_it_names() {
  let reads: Arc<TestLevelReads> = Arc::default();
  let (mut world, _, scene) = show(&reads);

  let weather: WorldLevelReload = WorldLevelReload {
    is_weather: true,
    ..WorldLevelReload::default()
  };

  assert_eq!(world.reload_scene("zaton", source(&reads), weather), Some(scene));
  advance_until_read(&mut world, &reads, [1, 2, 1, 1]);
  std::thread::sleep(QUIET);
  assert_eq!(reads.count(), [1, 2, 1, 1]);

  let spawn_and_lights: WorldLevelReload = WorldLevelReload {
    is_spawn: true,
    is_lights: true,
    ..WorldLevelReload::default()
  };

  assert_eq!(
    world.reload_scene("zaton", source(&reads), spawn_and_lights),
    Some(scene)
  );
  advance_until_read(&mut world, &reads, [2, 2, 2, 1]);
  std::thread::sleep(QUIET);
  assert_eq!(reads.count(), [2, 2, 2, 1]);

  let particles: WorldLevelReload = WorldLevelReload {
    is_particles: true,
    ..WorldLevelReload::default()
  };

  assert_eq!(world.reload_scene("zaton", source(&reads), particles), Some(scene));
  advance_until_read(&mut world, &reads, [2, 2, 2, 2]);
  std::thread::sleep(QUIET);
  assert_eq!(reads.count(), [2, 2, 2, 2]);
}

#[test]
fn reopening_a_scene_streams_it_afresh_for_its_viewers() {
  let reads: Arc<TestLevelReads> = Arc::default();
  let (mut world, viewport, scene) = show(&reads);
  let reopened: RenderSceneId = world.reopen_scene("zaton", source(&reads)).expect("the scene reopened");

  assert_ne!(reopened, scene);
  assert_eq!(world.get_asked_scene(viewport), Some(reopened));

  let frames: HashMap<RenderSceneId, RenderSceneFrame> = advance_until_read(&mut world, &reads, [2, 2, 2, 2]);

  assert_eq!(frames.keys().copied().collect::<Vec<_>>(), vec![reopened]);
  assert_eq!(frames[&reopened].driver, Some(viewport));
  assert_eq!(reads.count(), [2, 2, 2, 2]);
}

#[test]
fn a_key_no_scene_has_moves_nothing() {
  let reads: Arc<TestLevelReads> = Arc::default();
  let (mut world, _, _) = show(&reads);

  assert_eq!(
    world.reload_scene("jupiter", source(&reads), WorldLevelReload::default()),
    None
  );
  assert_eq!(world.reopen_scene("jupiter", source(&reads)), None);
}
