use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use xrf_anm::AnmFile;
use xrf_environment::WeatherDescriptor;
use xrf_error::{XrfError, XrfResult};
use xrf_material::XraySurfaceDescriptor;
use xrf_renderer::{
  RenderAssetSource, RenderLevelDetails, RenderLevelParticles, RenderLevelSource, RenderLevelSpawn, RenderLevelWeather,
  RenderSpawnModels,
};
use xrf_visual::{LightsDescription, SectorPackage};

use crate::contract::world_viewport_event::WorldViewportEvent;
use crate::world_event_sink::WorldEventSink;

/// How a test level packs a sector.
pub type TestSectorPack = Box<dyn Fn(u32) -> XrfResult<SectorPackage> + Send + Sync>;

/// How many times each read of a test level, or of the sources sharing its count, was asked for.
#[derive(Default)]
pub struct TestLevelReads {
  pub lights: AtomicU32,
  pub weather: AtomicU32,
  pub spawn: AtomicU32,
  pub particles: AtomicU32,
  pub details: AtomicU32,
}

impl TestLevelReads {
  /// The lights, weather, spawn, particles and grass reads asked for so far.
  pub fn count(&self) -> [u32; 5] {
    [&self.lights, &self.weather, &self.spawn, &self.particles, &self.details].map(|count| count.load(Ordering::SeqCst))
  }
}

/// A level holding nothing but what a test gives it: the key its scene is shared by, its sectors, each packed as
/// `pack` says, and its object motions by name; everything else it is asked for is not there, each ask counted.
pub struct TestLevelSource {
  pub key: Option<String>,
  pub sectors: u32,
  pub pack: TestSectorPack,
  pub motions: HashMap<String, AnmFile>,
  pub reads: Arc<TestLevelReads>,
}

impl Default for TestLevelSource {
  fn default() -> Self {
    Self {
      key: None,
      sectors: 0,
      pack: Box::new(|_| Err(Self::none())),
      motions: HashMap::new(),
      reads: Arc::default(),
    }
  }
}

impl TestLevelSource {
  /// What it answers for whatever it does not hold.
  pub fn none() -> XrfError {
    XrfError::new_not_found_error("nothing")
  }
}

impl RenderAssetSource for TestLevelSource {
  fn get_texture_scope(&self) -> &str {
    "test"
  }

  fn read_texture(&self, _: &str) -> XrfResult<Option<Vec<u8>>> {
    Ok(None)
  }
}

impl RenderLevelSource for TestLevelSource {
  fn get_sector_count(&self) -> u32 {
    self.sectors
  }

  fn get_scene_key(&self) -> Option<String> {
    self.key.clone()
  }

  fn pack_sector(&self, sector: u32) -> XrfResult<SectorPackage> {
    (self.pack)(sector)
  }

  fn get_surfaces(&self) -> &[XraySurfaceDescriptor] {
    &[]
  }

  fn read_lights(&self) -> XrfResult<LightsDescription> {
    self.reads.lights.fetch_add(1, Ordering::SeqCst);

    Err(Self::none())
  }

  fn read_weather(&self) -> XrfResult<RenderLevelWeather> {
    self.reads.weather.fetch_add(1, Ordering::SeqCst);

    Err(Self::none())
  }

  fn read_weather_cycle(&self, _: &str) -> XrfResult<Vec<WeatherDescriptor>> {
    Err(Self::none())
  }

  fn read_spawn(&self) -> XrfResult<RenderLevelSpawn> {
    self.reads.spawn.fetch_add(1, Ordering::SeqCst);

    Err(Self::none())
  }

  fn read_spawn_models(&self, _: &[String]) -> XrfResult<RenderSpawnModels> {
    Err(Self::none())
  }

  fn read_details(&self) -> XrfResult<Option<RenderLevelDetails>> {
    self.reads.details.fetch_add(1, Ordering::SeqCst);

    Ok(None)
  }

  fn read_particles(&self) -> XrfResult<Option<RenderLevelParticles>> {
    self.reads.particles.fetch_add(1, Ordering::SeqCst);

    Ok(None)
  }

  fn read_object_motion(&self, name: &str) -> XrfResult<AnmFile> {
    self.motions.get(name).cloned().ok_or_else(Self::none)
  }
}

/// A viewport's world events sent nowhere, for a test attaching a viewport only to show it a level.
pub struct SilentSink;

impl WorldEventSink for SilentSink {
  fn send(&self, _: WorldViewportEvent) {}
}
