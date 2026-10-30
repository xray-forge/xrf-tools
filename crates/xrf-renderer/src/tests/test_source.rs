use std::collections::HashMap;
use std::sync::Mutex;

use xrf_environment::WeatherDescriptor;
use xrf_error::{XrfError, XrfResult};
use xrf_material::XraySurfaceDescriptor;
use xrf_visual::{LightsDescription, SectorPackage};

use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_cache_store::RenderCacheStore;
use crate::host::render_level_details::RenderLevelDetails;
use crate::host::render_level_particles::RenderLevelParticles;
use crate::host::render_level_source::RenderLevelSource;
use crate::host::render_level_spawn::RenderLevelSpawn;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::host::render_spawn_models::RenderSpawnModels;
use crate::host::render_tree_type::RenderTreeType;

/// A source holding no textures: every request still takes a slot, which loads as missing.
pub struct EmptySource;

impl RenderAssetSource for EmptySource {
  fn get_texture_scope(&self) -> &str {
    "test"
  }

  fn read_texture(&self, _: &str) -> XrfResult<Option<Vec<u8>>> {
    Ok(None)
  }
}

/// A level of nothing but its tree types: no sectors, no textures, and nothing else to read.
pub struct TreeTypesSource {
  pub types: Vec<RenderTreeType>,
}

impl TreeTypesSource {
  fn none() -> XrfError {
    XrfError::new_not_found_error("The test level has nothing else")
  }
}

impl RenderAssetSource for TreeTypesSource {
  fn get_texture_scope(&self) -> &str {
    "test"
  }

  fn read_texture(&self, _: &str) -> XrfResult<Option<Vec<u8>>> {
    Ok(None)
  }
}

impl RenderLevelSource for TreeTypesSource {
  fn get_sector_count(&self) -> u32 {
    0
  }

  fn pack_sector(&self, _: u32) -> XrfResult<SectorPackage> {
    Err(Self::none())
  }

  fn get_surfaces(&self) -> &[XraySurfaceDescriptor] {
    &[]
  }

  fn read_tree_types(&self) -> XrfResult<Vec<RenderTreeType>> {
    Ok(self.types.clone())
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
}

/// A cache kept in memory, by name.
#[derive(Default)]
pub struct MemoryCacheStore {
  pub entries: Mutex<HashMap<String, Vec<u8>>>,
}

impl RenderCacheStore for MemoryCacheStore {
  fn read_cached(&self, name: &str) -> Option<Vec<u8>> {
    self.entries.lock().unwrap().get(name).cloned()
  }

  fn write_cached(&self, name: &str, bytes: &[u8]) {
    self.entries.lock().unwrap().insert(name.to_owned(), bytes.to_vec());
  }
}
