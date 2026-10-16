use std::collections::HashMap;

use xrf_anm::AnmFile;
use xrf_environment::WeatherDescriptor;
use xrf_error::{XrfError, XrfResult};
use xrf_material::XraySurfaceDescriptor;
use xrf_renderer::{
  RenderAssetSource, RenderLevelDetails, RenderLevelParticles, RenderLevelSource, RenderLevelSpawn, RenderLevelWeather,
  RenderSpawnModels,
};
use xrf_visual::{LightsDescription, SectorPackage};

/// How a test level packs a sector.
pub type TestSectorPack = Box<dyn Fn(u32) -> XrfResult<SectorPackage> + Send + Sync>;

/// A level holding nothing but what a test gives it: the key its scene is shared by, its sectors, each packed as
/// `pack` says, and its object motions by name; everything else it is asked for is not there.
pub struct TestLevelSource {
  pub key: Option<String>,
  pub sectors: u32,
  pub pack: TestSectorPack,
  pub motions: HashMap<String, AnmFile>,
}

impl Default for TestLevelSource {
  fn default() -> Self {
    Self {
      key: None,
      sectors: 0,
      pack: Box::new(|_| Err(Self::none())),
      motions: HashMap::new(),
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
  fn get_texture_scope(&self) -> String {
    String::from("test")
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
    Err(Self::none())
  }

  fn read_weather(&self) -> XrfResult<RenderLevelWeather> {
    Err(Self::none())
  }

  fn read_weather_cycle(&self, _: &str) -> XrfResult<Vec<WeatherDescriptor>> {
    Err(Self::none())
  }

  fn read_spawn(&self) -> XrfResult<RenderLevelSpawn> {
    Err(Self::none())
  }

  fn read_spawn_models(&self, _: &[String]) -> XrfResult<RenderSpawnModels> {
    Err(Self::none())
  }

  fn read_details(&self) -> XrfResult<Option<RenderLevelDetails>> {
    Ok(None)
  }

  fn read_particles(&self) -> XrfResult<Option<RenderLevelParticles>> {
    Ok(None)
  }

  fn read_object_motion(&self, name: &str) -> XrfResult<AnmFile> {
    self.motions.get(name).cloned().ok_or_else(Self::none)
  }
}
