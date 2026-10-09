//! The scenes the viewports show: one a level opening, shared by every viewport showing it, driven by the first.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use xrf_environment::WeatherDescriptor;
use xrf_error::{XrfError, XrfResult};
use xrf_material::XraySurfaceDescriptor;
use xrf_renderer::{
  RenderAssetSource, RenderLevelDetails, RenderLevelParticles, RenderLevelSource, RenderLevelSpawn, RenderLevelWeather,
  RenderSceneFrame, RenderSceneId, RenderSpawnModels, RenderViewportId, RenderWorld,
};
use xrf_visual::{LightsDescription, SectorPackage};

use crate::tests::test_workers::create_workers;
use crate::world::World;

/// A level with nothing in it, shared with every other source of the same key.
struct EmptySource {
  key: Option<String>,
}

impl RenderAssetSource for EmptySource {
  fn get_texture_scope(&self) -> String {
    String::from("test")
  }

  fn read_texture(&self, _: &str) -> XrfResult<Option<Vec<u8>>> {
    Ok(None)
  }
}

impl RenderLevelSource for EmptySource {
  fn get_sector_count(&self) -> u32 {
    0
  }

  fn get_scene_key(&self) -> Option<String> {
    self.key.clone()
  }

  fn pack_sector(&self, _: u32) -> XrfResult<SectorPackage> {
    Err(none())
  }

  fn get_surfaces(&self) -> &[XraySurfaceDescriptor] {
    &[]
  }

  fn read_lights(&self) -> XrfResult<LightsDescription> {
    Err(none())
  }

  fn read_weather(&self) -> XrfResult<RenderLevelWeather> {
    Err(none())
  }

  fn read_weather_cycle(&self, _: &str) -> XrfResult<Vec<WeatherDescriptor>> {
    Err(none())
  }

  fn read_spawn(&self) -> XrfResult<RenderLevelSpawn> {
    Err(none())
  }

  fn read_spawn_models(&self, _: &[String]) -> XrfResult<RenderSpawnModels> {
    Err(none())
  }

  fn read_details(&self) -> XrfResult<Option<RenderLevelDetails>> {
    Ok(None)
  }

  fn read_particles(&self) -> XrfResult<Option<RenderLevelParticles>> {
    Ok(None)
  }
}

fn none() -> XrfError {
  XrfError::new_not_found_error("nothing")
}

fn source(key: Option<&str>) -> Arc<dyn RenderLevelSource> {
  Arc::new(EmptySource {
    key: key.map(String::from),
  })
}

/// Each scene the world plays this frame, by the viewport driving it.
fn list_drivers(world: &mut World) -> HashMap<RenderSceneId, Option<RenderViewportId>> {
  world
    .advance_scenes(HashMap::new())
    .into_iter()
    .map(|(id, frame): (RenderSceneId, RenderSceneFrame)| (id, frame.driver))
    .collect()
}

#[test]
fn viewports_showing_one_opening_share_its_scene_driven_by_the_first() {
  let mut world: World = World::new(create_workers());
  let (first, second) = (RenderViewportId(1), RenderViewportId(2));

  world.show_level(first, Some(source(Some("zaton"))));
  world.show_level(second, Some(source(Some("zaton"))));

  let drivers: HashMap<RenderSceneId, Option<RenderViewportId>> = list_drivers(&mut world);

  assert_eq!(drivers.len(), 1);
  assert_eq!(drivers.values().next(), Some(&Some(first)));

  // The first goes: the second drives the same scene on.
  world.detach(first);

  let after: HashMap<RenderSceneId, Option<RenderViewportId>> = list_drivers(&mut world);

  assert_eq!(after.keys().collect::<Vec<_>>(), drivers.keys().collect::<Vec<_>>());
  assert_eq!(after.values().next(), Some(&Some(second)));

  // The last goes, and the scene with it.
  world.detach(second);
  assert!(list_drivers(&mut world).is_empty());
}

#[test]
fn sources_sharing_nothing_stream_scenes_of_their_own() {
  let mut world: World = World::new(create_workers());
  let (first, second) = (RenderViewportId(1), RenderViewportId(2));

  world.show_level(first, Some(source(None)));
  world.show_level(second, Some(source(None)));

  assert_eq!(list_drivers(&mut world).len(), 2);
}

#[test]
fn showing_another_level_leaves_the_last_scene() {
  let mut world: World = World::new(create_workers());
  let viewport: RenderViewportId = RenderViewportId(1);

  world.show_level(viewport, Some(source(Some("zaton"))));

  let before: Vec<RenderSceneId> = list_drivers(&mut world).into_keys().collect();

  world.show_level(viewport, Some(source(Some("jupiter"))));

  let after: Vec<RenderSceneId> = list_drivers(&mut world).into_keys().collect();

  assert_eq!(after.len(), 1);
  assert_ne!(after, before);

  world.show_level(viewport, None);
  assert!(list_drivers(&mut world).is_empty());
}

#[test]
fn restarting_the_scenes_streams_each_again_under_its_id_and_driver() {
  let mut world: World = World::new(create_workers());
  let viewport: RenderViewportId = RenderViewportId(1);

  world.show_level(viewport, Some(source(Some("zaton"))));

  let before: HashMap<RenderSceneId, RenderSceneFrame> = world.advance_scenes(HashMap::new());

  // So the restarted streaming's start is later than the first's on any clock.
  std::thread::sleep(Duration::from_millis(1));
  world.restart_scenes();

  let after: HashMap<RenderSceneId, RenderSceneFrame> = world.advance_scenes(HashMap::new());

  assert_eq!(after.keys().collect::<Vec<_>>(), before.keys().collect::<Vec<_>>());

  for (id, frame) in &after {
    assert_eq!(frame.driver, Some(viewport));
    assert!(frame.streaming.started > before[id].streaming.started);
  }
}
