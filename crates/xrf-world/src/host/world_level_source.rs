use xrf_anm::AnmFile;
use xrf_environment::WeatherDescriptor;
use xrf_error::{XrfError, XrfResult};
use xrf_renderer::{
  RenderLevelDetails, RenderLevelHelpers, RenderLevelParticles, RenderLevelSource, RenderLevelSpawn, RenderMotion,
  RenderSpawnModels,
};
use xrf_visual::{LightsDescription, SectorPackage};

use crate::host::world_level_weather::WorldLevelWeather;

/// An open level as the world plays it: the application reads its parts on demand, on the renderer's loader threads,
/// and the world hands what the scene draws to the renderer, which reads the rest through [`RenderLevelSource`].
pub trait WorldLevelSource: RenderLevelSource {
  /// What every source of the same level names, a next source of it included, so the viewports showing it share one
  /// scene and a next source moves onto it; none for a source no other shares, as a model posed for one viewport is.
  fn get_scene_key(&self) -> Option<String> {
    None
  }

  /// One sector's geometry, packed, called from loader threads side by side.
  ///
  /// # Errors
  ///
  /// Returns an error when the sector cannot be packed.
  fn pack_sector(&self, sector: u32) -> XrfResult<SectorPackage>;

  /// The level's lights, its own and its spawned lamps', called from a loader thread as the level opens and again for a
  /// next source that reloads them; their projectors are read through
  /// [`xrf_renderer::RenderAssetSource::read_texture`] after.
  ///
  /// # Errors
  ///
  /// Returns an error when they cannot be collected.
  fn read_lights(&self) -> XrfResult<LightsDescription>;

  /// What every weather of the level plays with, called from a loader thread as a viewport shows the level and again
  /// for a next source that reloads it; its skies are read through
  /// [`xrf_renderer::RenderAssetSource::read_texture`] after.
  ///
  /// # Errors
  ///
  /// Returns an error when the game's environment configs cannot be read.
  fn read_weather(&self) -> XrfResult<WorldLevelWeather>;

  /// One cycle of the level's game by name, its keyframes sorted by time, called from a loader thread.
  ///
  /// # Errors
  ///
  /// Returns an error for a cycle the game does not have, or configs that cannot be read.
  fn read_weather_cycle(&self, name: &str) -> XrfResult<Vec<WeatherDescriptor>>;

  /// The objects of the level's spawn the viewer draws, called from a loader thread as the level opens and again for a
  /// next source that reloads them.
  ///
  /// # Errors
  ///
  /// Returns an error when the level's spawn cannot be read.
  fn read_spawn(&self) -> XrfResult<RenderLevelSpawn>;

  /// The level objects the viewer draws helpers for, called from a loader thread as the level opens and again with
  /// the spawn for a next source that reloads it; none for a source with no level objects.
  ///
  /// # Errors
  ///
  /// Returns an error when the level's objects cannot be read.
  fn read_helpers(&self) -> XrfResult<RenderLevelHelpers> {
    Ok(RenderLevelHelpers::default())
  }

  /// A batch of the visuals [`WorldLevelSource::read_spawn`] named, posed, and how the level lights the objects
  /// standing as them, called from loader threads; a visual that cannot be read is left out, and reported by the
  /// application.
  ///
  /// # Errors
  ///
  /// Returns an error when none of them can be read for a reason they share.
  fn read_spawn_models(&self, names: &[String]) -> XrfResult<RenderSpawnModels>;

  /// The level's grass, its slots planted onto its collision form and its models dressed, called from a loader thread
  /// as the level opens and again for a next source that reloads it; none for a level without a detail library.
  ///
  /// # Errors
  ///
  /// Returns an error when the library or the collision form cannot be read.
  fn read_details(&self) -> XrfResult<Option<RenderLevelDetails>>;

  /// The level's particle systems, planted and played by its zones, called from a loader thread as the level opens and
  /// again for a next source that reloads them; none for a source with no level.
  ///
  /// # Errors
  ///
  /// Returns an error when the game's particle library cannot be read.
  fn read_particles(&self) -> XrfResult<Option<RenderLevelParticles>>;

  /// One motion of the skinned model [`WorldLevelSource::read_spawn_models`] gave a skeleton, baked, called from a
  /// loader thread; a level's spawned models are posed already and have none.
  ///
  /// # Errors
  ///
  /// Returns an error for a motion the model does not have, or files that cannot be read.
  fn read_motion(&self, name: &str) -> XrfResult<RenderMotion> {
    Err(XrfError::new_not_found_error(format!("No motion '{name}' to play")))
  }

  /// An object motion a spawned object moves along (`CObjectAnimator::Load`), by the name its spawn gives, called from
  /// a loader thread.
  ///
  /// # Errors
  ///
  /// Returns an error for a motion that cannot be found or read.
  fn read_object_motion(&self, name: &str) -> XrfResult<AnmFile> {
    Err(XrfError::new_not_found_error(format!(
      "No object motion '{name}' to play"
    )))
  }
}
