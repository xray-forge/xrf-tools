use std::path::Path;

use byteorder::ByteOrder;
use xrf_editable_level::EditableLevelGuid;
use xrf_error::XrfResult;

use crate::level::scene::level_decompile_options::LevelDecompileOptions;
use crate::level::scene::level_scene_decompile::LevelSceneDecompile;
use crate::level::scene::level_scene_draft::LevelSceneDraft;
use crate::level::scene::level_scene_listing::LevelSceneListing;
use crate::level::scene::level_scene_sources::LevelSceneSources;
use crate::level::scene::steps::level_bundle_step::LevelBundleStep;
use crate::level::scene::steps::level_companion_step::LevelCompanionStep;
use crate::level::scene::steps::level_detail_step::LevelDetailStep;
use crate::level::scene::steps::level_game_step::LevelGameStep;
use crate::level::scene::steps::level_geometry_step::LevelGeometryStep;
use crate::level::scene::steps::level_map_step::LevelMapStep;
use crate::level::scene::steps::level_root_spawn::LevelRootSpawn;
use crate::level::scene::steps::level_scene_step::LevelSceneStep;

/// Decompiles a level's directory into a Level Editor scene, step by step over one draft: the map's settings, the
/// geometry and its objects,
/// sectors and portals; the bundle's glows and lights; the files the build passes through from a part
/// (`BuilderGame.cpp`); the details; and the game's ways and spawn points. Each step names the files it reads; the decompile reads
/// them all once and hashes them into the scene's GUID, so decompiling the same level twice gives the same scene
/// (decision 22).
pub struct LevelSceneDecompiler;

impl LevelSceneDecompiler {
  /// The owner the scene's `[level_tag]` names.
  pub const OWNER: &'static str = LevelSceneDraft::OWNER;

  /// The files of a level's directory the decompile reads, every step's, in the order the steps run.
  pub fn get_source_files() -> Vec<&'static str> {
    let mut files: Vec<&'static str> = [
      LevelMapStep::FILES,
      LevelGeometryStep::FILES,
      LevelBundleStep::FILES,
      LevelCompanionStep::FILES,
      LevelDetailStep::FILES,
      LevelGameStep::FILES,
    ]
    .concat();
    let mut seen: Vec<&'static str> = Vec::with_capacity(files.len());

    files.retain(|file| {
      let is_new: bool = !seen.contains(file);

      seen.push(file);
      is_new
    });
    files
  }

  /// The files every step reads by what their names hold.
  fn get_listings() -> Vec<LevelSceneListing> {
    [
      LevelMapStep::LISTINGS,
      LevelGeometryStep::LISTINGS,
      LevelBundleStep::LISTINGS,
      LevelCompanionStep::LISTINGS,
      LevelDetailStep::LISTINGS,
      LevelGameStep::LISTINGS,
    ]
    .concat()
  }

  /// The scene of the level in `directory`: a part per tool whose compiled file the directory holds, the objects of
  /// its geometry, and the main file its name and those files give.
  ///
  /// # Errors
  ///
  /// Returns an error when the directory holds none of [`Self::get_source_files`], one cannot be read or is not the
  /// file its name says, the root spawn given cannot be read or holds no objects chunk or graph, or a step refuses what
  /// it holds.
  pub fn decompile<T: ByteOrder>(directory: &Path, options: &LevelDecompileOptions) -> XrfResult<LevelSceneDecompile> {
    let mut sources: LevelSceneSources =
      LevelSceneSources::read(directory, &Self::get_source_files(), &Self::get_listings())?;
    let root_spawn: Option<LevelRootSpawn> = options
      .root_spawn
      .as_deref()
      .map(|path| LevelRootSpawn::read::<T>(path, directory))
      .transpose()?;
    let guid: EditableLevelGuid = sources.get_guid(root_spawn.as_ref().and_then(LevelRootSpawn::get_guid_hash));
    let mut draft: LevelSceneDraft = LevelSceneDraft::new(guid, &options.name, root_spawn);

    LevelMapStep::decompile::<T>(&mut sources, options, &mut draft)?;
    LevelGeometryStep::decompile::<T>(&mut sources, options, &mut draft)?;
    LevelBundleStep::decompile::<T>(&mut sources, options, &mut draft)?;
    LevelCompanionStep::decompile::<T>(&mut sources, options, &mut draft)?;
    LevelDetailStep::decompile::<T>(&mut sources, options, &mut draft)?;
    LevelGameStep::decompile::<T>(&mut sources, options, &mut draft)?;

    Ok(draft.finish(options, &sources))
  }
}
