//! A scene moved onto a next source of its level: it keeps its id and viewers, and reads again only what it is told.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_renderer::{
  RenderLevelSource, RenderSceneFrame, RenderSceneId, RenderSceneUpdate, RenderStreamingProgress, RenderViewportId,
  RenderWorld,
};

use crate::contract::world_level_reload::WorldLevelReload;
use crate::host::world_level_source::WorldLevelSource;
use crate::tests::fixtures::{SilentSink, TestLevelReads, TestLevelSource};
use crate::tests::test_workers::create_workers;
use crate::world::World;

/// Longer than a test level's reads take on the test workers.
const WAIT: Duration = Duration::from_secs(10);

/// What a reload is given time to start reading before a test says it read nothing.
const QUIET: Duration = Duration::from_millis(100);

/// A source of the level `zaton` counting its reads into `reads`.
fn source(reads: &Arc<TestLevelReads>) -> Arc<dyn WorldLevelSource> {
  Arc::new(TestLevelSource {
    key: Some(String::from("zaton")),
    reads: Arc::clone(reads),
    ..TestLevelSource::default()
  })
}

/// Whether every read of the level's packages its scene waits on has answered.
fn is_answered(progress: &RenderStreamingProgress) -> bool {
  progress.is_spawn_done && progress.is_lights_done && progress.is_grass_done && progress.is_particles_done
}

/// Advances the world's scenes until `reads` has counted `expected` and each read has answered, answering the last
/// frames and every update posted meanwhile.
fn advance_until_read(
  world: &mut World,
  reads: &TestLevelReads,
  expected: [u32; 5],
) -> (HashMap<RenderSceneId, RenderSceneFrame>, Vec<RenderSceneUpdate>) {
  let started: Instant = Instant::now();
  let mut posted: Vec<RenderSceneUpdate> = Vec::new();

  loop {
    let mut frames: HashMap<RenderSceneId, RenderSceneFrame> = world.advance_scenes(Instant::now(), HashMap::new());
    let is_answered: bool = frames.values().all(|frame| is_answered(&frame.streaming));

    for frame in frames.values_mut() {
      posted.append(&mut frame.updates);
    }

    if (reads.count() == expected && is_answered) || started.elapsed() > WAIT {
      return (frames, posted);
    }

    std::thread::sleep(Duration::from_millis(2));
  }
}

/// Whether `updates` replace the scene's grass.
fn is_grass_posted(updates: &[RenderSceneUpdate]) -> bool {
  updates
    .iter()
    .any(|update| matches!(update, RenderSceneUpdate::AddGrass(_)))
}

/// A world with one viewport showing the level, its first reads done: lights, weather, spawn, particles and grass once
/// each.
fn show(reads: &Arc<TestLevelReads>) -> (World, RenderViewportId, RenderSceneId) {
  let mut world: World = World::new(create_workers());
  let viewport: RenderViewportId = RenderViewportId(1);

  world.attach(viewport, Box::new(SilentSink));

  let scene: RenderSceneId = world
    .show_level(viewport, Some(source(reads)))
    .expect("the level shown");
  let (_, posted) = advance_until_read(&mut world, reads, [1, 1, 1, 1, 1]);

  assert_eq!(reads.count(), [1, 1, 1, 1, 1]);
  assert!(is_grass_posted(&posted), "the grass is posted once read");

  (world, viewport, scene)
}

#[test]
fn a_next_source_keeps_the_scene_its_viewers_and_everything_drawn() {
  let reads: Arc<TestLevelReads> = Arc::default();
  let (mut world, viewport, scene) = show(&reads);
  let next: Arc<dyn WorldLevelSource> = source(&reads);

  assert_eq!(
    world.reload_scene("zaton", Arc::clone(&next), WorldLevelReload::default()),
    Some(scene)
  );
  assert_eq!(world.get_asked_scene(viewport), Some(scene));

  std::thread::sleep(QUIET);

  let frames: HashMap<RenderSceneId, RenderSceneFrame> = world.advance_scenes(Instant::now(), HashMap::new());

  assert_eq!(frames.keys().copied().collect::<Vec<_>>(), vec![scene]);
  assert!(Arc::ptr_eq(
    &frames[&scene].source,
    &(Arc::clone(&next) as Arc<dyn RenderLevelSource>)
  ));
  assert!(frames[&scene].updates.is_empty());
  // An unaffected commit reads nothing again.
  assert_eq!(reads.count(), [1, 1, 1, 1, 1]);
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
  advance_until_read(&mut world, &reads, [1, 2, 1, 1, 1]);
  std::thread::sleep(QUIET);
  assert_eq!(reads.count(), [1, 2, 1, 1, 1]);

  let spawn_and_lights: WorldLevelReload = WorldLevelReload {
    is_spawn: true,
    is_lights: true,
    ..WorldLevelReload::default()
  };

  assert_eq!(
    world.reload_scene("zaton", source(&reads), spawn_and_lights),
    Some(scene)
  );
  advance_until_read(&mut world, &reads, [2, 2, 2, 1, 1]);
  std::thread::sleep(QUIET);
  assert_eq!(reads.count(), [2, 2, 2, 1, 1]);

  let particles: WorldLevelReload = WorldLevelReload {
    is_particles: true,
    ..WorldLevelReload::default()
  };

  assert_eq!(world.reload_scene("zaton", source(&reads), particles), Some(scene));

  let (_, posted) = advance_until_read(&mut world, &reads, [2, 2, 2, 2, 1]);

  std::thread::sleep(QUIET);
  assert_eq!(reads.count(), [2, 2, 2, 2, 1]);
  // A reload not naming the grass leaves it as drawn.
  assert!(!is_grass_posted(&posted));
}

#[test]
fn a_grass_reload_reads_the_grass_alone_and_replaces_it() {
  let reads: Arc<TestLevelReads> = Arc::default();
  let (mut world, viewport, scene) = show(&reads);
  let grass: WorldLevelReload = WorldLevelReload {
    is_grass: true,
    ..WorldLevelReload::default()
  };

  assert_eq!(world.reload_scene("zaton", source(&reads), grass), Some(scene));

  let (frames, mut posted) = advance_until_read(&mut world, &reads, [1, 1, 1, 1, 2]);

  std::thread::sleep(QUIET);

  let after: HashMap<RenderSceneId, RenderSceneFrame> = world.advance_scenes(Instant::now(), HashMap::new());

  posted.extend(after.into_values().flat_map(|frame| frame.updates));

  assert_eq!(reads.count(), [1, 1, 1, 1, 2]);
  assert_eq!(frames.keys().copied().collect::<Vec<_>>(), vec![scene]);
  assert_eq!(world.get_asked_scene(viewport), Some(scene));
  // The grass alone is posted again, nothing else taken out or read.
  assert!(
    matches!(posted.as_slice(), [RenderSceneUpdate::AddGrass(None)]),
    "posted {} updates",
    posted.len()
  );
}

#[test]
fn reopening_a_scene_streams_it_afresh_for_its_viewers() {
  let reads: Arc<TestLevelReads> = Arc::default();
  let (mut world, viewport, scene) = show(&reads);
  let reopened: RenderSceneId = world.reopen_scene("zaton", source(&reads)).expect("the scene reopened");

  assert_ne!(reopened, scene);
  assert_eq!(world.get_asked_scene(viewport), Some(reopened));

  let (frames, _) = advance_until_read(&mut world, &reads, [2, 2, 2, 2, 2]);

  assert_eq!(frames.keys().copied().collect::<Vec<_>>(), vec![reopened]);
  assert_eq!(frames[&reopened].driver, Some(viewport));
  assert_eq!(reads.count(), [2, 2, 2, 2, 2]);
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
