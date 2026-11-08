use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use xrf_anm::AnmFile;
use xrf_environment::WeatherDescriptor;
use xrf_error::{XrfError, XrfResult};
use xrf_material::XraySurfaceDescriptor;
use xrf_renderer::{
  RenderAssetSource, RenderLevelDetails, RenderLevelHelpers, RenderLevelParticles, RenderLevelSource, RenderLevelSpawn,
  RenderSpawnModels,
};
use xrf_visual::{LightsDescription, SectorPackage};

use crate::contract::world_viewport_event::WorldViewportEvent;
use crate::host::sound::level::world_level_sounds::WorldLevelSounds;
use crate::host::sound::sink::world_sound_sink::WorldSoundSink;
use crate::host::world_event_sink::WorldEventSink;
use crate::host::world_level_source::WorldLevelSource;
use crate::host::world_level_weather::WorldLevelWeather;

/// How a test level packs a sector.
pub(crate) type TestSectorPack = Box<dyn Fn(u32) -> XrfResult<SectorPackage> + Send + Sync>;

/// How many times each read of a test level, or of the sources sharing its count, was asked for.
#[derive(Default)]
pub(crate) struct TestLevelReads {
  pub lights: AtomicU32,
  pub weather: AtomicU32,
  pub spawn: AtomicU32,
  pub particles: AtomicU32,
  pub details: AtomicU32,
}

impl TestLevelReads {
  /// The lights, weather, spawn, particles and grass reads asked for so far.
  pub(crate) fn count(&self) -> [u32; 5] {
    [&self.lights, &self.weather, &self.spawn, &self.particles, &self.details].map(|count| count.load(Ordering::SeqCst))
  }
}

/// A level holding nothing but what a test gives it: the key its scene is shared by, its sectors, each packed as
/// `pack` says, its object motions by name, its sounds and where their frames go; everything else it is asked for is
/// not there, each ask counted.
pub(crate) struct TestLevelSource {
  pub key: Option<String>,
  pub sectors: u32,
  pub pack: TestSectorPack,
  pub motions: HashMap<String, AnmFile>,
  pub sounds: WorldLevelSounds,
  /// The sink the scene's sound frames go to, taken by the first scene asking.
  pub sound_sink: Mutex<Option<Box<dyn WorldSoundSink>>>,
  pub reads: Arc<TestLevelReads>,
  /// How long reading its helpers takes, standing for a worker a loaded machine is slow to run: nothing waits on the
  /// helpers, so they can be posted after every part a scene waits on has answered.
  pub helpers_delay: Duration,
}

impl Default for TestLevelSource {
  fn default() -> Self {
    Self {
      key: None,
      sectors: 0,
      pack: Box::new(|_| Err(Self::none())),
      motions: HashMap::new(),
      sounds: WorldLevelSounds::default(),
      sound_sink: Mutex::new(None),
      reads: Arc::default(),
      helpers_delay: Duration::ZERO,
    }
  }
}

impl TestLevelSource {
  /// What it answers for whatever it does not hold.
  pub(crate) fn none() -> XrfError {
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

  fn get_surfaces(&self) -> &[XraySurfaceDescriptor] {
    &[]
  }
}

impl WorldLevelSource for TestLevelSource {
  fn get_scene_key(&self) -> Option<String> {
    self.key.clone()
  }

  fn pack_sector(&self, sector: u32) -> XrfResult<SectorPackage> {
    (self.pack)(sector)
  }

  fn read_lights(&self) -> XrfResult<LightsDescription> {
    self.reads.lights.fetch_add(1, Ordering::SeqCst);

    Err(Self::none())
  }

  fn read_weather(&self) -> XrfResult<WorldLevelWeather> {
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

  /// No level objects, as the source's own default answers, but only once `helpers_delay` has passed.
  fn read_helpers(&self) -> XrfResult<RenderLevelHelpers> {
    std::thread::sleep(self.helpers_delay);

    Ok(RenderLevelHelpers::default())
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

  fn read_sounds(&self) -> XrfResult<WorldLevelSounds> {
    Ok(self.sounds.clone())
  }

  fn open_sound(&self) -> Option<Box<dyn WorldSoundSink>> {
    self.sound_sink.lock().unwrap_or_else(PoisonError::into_inner).take()
  }
}

/// A viewport's world events sent nowhere, for a test attaching a viewport only to show it a level.
pub(crate) struct SilentSink;

impl WorldEventSink for SilentSink {
  fn send(&self, _: WorldViewportEvent) {}
}
