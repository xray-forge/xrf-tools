use xrf_environment::WeatherDescriptor;
use xrf_error::XrfResult;
use xrf_material::XraySurfaceDescriptor;
use xrf_visual::{LightsDescription, SectorPackage};

use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_level_details::RenderLevelDetails;
use crate::host::render_level_spawn::RenderLevelSpawn;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::host::render_spawn_models::RenderSpawnModels;

/// An open level as the renderer draws it: the application packs its sectors on demand, on the renderer's loader
/// threads, and reads the files they name.
pub trait RenderLevelSource: RenderAssetSource {
  /// How many sectors the level has.
  fn get_sector_count(&self) -> u32;

  /// One sector's geometry, packed, called from loader threads side by side.
  ///
  /// # Errors
  ///
  /// Returns an error when the sector cannot be packed.
  fn pack_sector(&self, sector: u32) -> XrfResult<SectorPackage>;

  /// How each entry of the level's shader table is drawn, in its order: what a sector's surface names by id.
  fn get_surfaces(&self) -> &[XraySurfaceDescriptor];

  /// The level's lights, its own and its spawned lamps', called once from a loader thread; their projectors are read
  /// through [`RenderAssetSource::read_texture`] after.
  ///
  /// # Errors
  ///
  /// Returns an error when they cannot be collected.
  fn read_lights(&self) -> XrfResult<LightsDescription>;

  /// What every weather of the level plays with, called once from a loader thread; its skies are read through
  /// [`RenderAssetSource::read_texture`] after.
  ///
  /// # Errors
  ///
  /// Returns an error when the game's environment configs cannot be read.
  fn read_weather(&self) -> XrfResult<RenderLevelWeather>;

  /// One cycle of the level's game by name, its keyframes sorted by time, called from a loader thread.
  ///
  /// # Errors
  ///
  /// Returns an error for a cycle the game does not have, or configs that cannot be read.
  fn read_weather_cycle(&self, name: &str) -> XrfResult<Vec<WeatherDescriptor>>;

  /// The objects of the level's spawn the viewer draws, called once from a loader thread.
  ///
  /// # Errors
  ///
  /// Returns an error when the level's spawn cannot be read.
  fn read_spawn(&self) -> XrfResult<RenderLevelSpawn>;

  /// A batch of the visuals [`RenderLevelSource::read_spawn`] named, posed, and how the level lights the objects
  /// standing as them, called from loader threads; a visual that cannot be read is left out, and reported by the
  /// application.
  ///
  /// # Errors
  ///
  /// Returns an error when none of them can be read for a reason they share.
  fn read_spawn_models(&self, names: &[String]) -> XrfResult<RenderSpawnModels>;

  /// The level's grass, its slots planted onto its collision form and its models dressed, called once from a loader
  /// thread; none for a level without a detail library.
  ///
  /// # Errors
  ///
  /// Returns an error when the library or the collision form cannot be read.
  fn read_details(&self) -> XrfResult<Option<RenderLevelDetails>>;
}
