use std::path::{Path, PathBuf};

use rayon::prelude::*;
use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};

use crate::spawn::spawn_game_level::SpawnGameLevel;
use crate::spawn::spawn_game_levels::SpawnGameLevels;
use crate::spawn::spawn_level_files::SpawnLevelFiles;
use crate::spawn::spawn_root_build::SpawnRootBuild;
use crate::spawn::spawn_root_builder::SpawnRootBuilder;
use crate::spawn::spawn_root_level::SpawnRootLevel;
use crate::spawn::spawn_root_outcome::SpawnRootOutcome;
use crate::spawn::spawn_root_request::SpawnRootRequest;

/// The root spawn build over a gamedata root, as `xrAI -s` runs over `$game_levels$` and writes `$game_spawn$`: the
/// levels the game lists read, each level's graph built (in parallel; the result is the same), the spawn built and
/// written.
pub struct SpawnRootGamedata;

impl SpawnRootGamedata {
  /// The game's own spawn, which a build writes over only when forced.
  pub const GAME_SPAWN: &'static str = "all";
  const LEVELS_DIRECTORY: &'static str = "levels";
  const SPAWNS_DIRECTORY: &'static str = "spawns";

  /// Builds the spawn `request` asks for and writes it.
  ///
  /// # Errors
  ///
  /// Returns an error when the level list does not read, names no level to build or not one asked for, a level's
  /// files do not read, the build refuses ([`SpawnRootBuilder::build`]), the game's own spawn is there and the request
  /// is not forced, or the spawn cannot be written.
  pub fn build(request: &SpawnRootRequest) -> XrfResult<SpawnRootOutcome> {
    let destination: PathBuf = request
      .gamedata
      .join(Self::SPAWNS_DIRECTORY)
      .join(format!("{}.spawn", request.name));

    if request.name == Self::GAME_SPAWN && destination.exists() && !request.is_forced {
      return Err(XrfError::new_invalid_error(format!(
        "{} is the game's own spawn; force the build to write over it",
        destination.display()
      )));
    }

    let listed: SpawnGameLevels = SpawnGameLevels::read_from_path(&request.gamedata.join(SpawnGameLevels::PATH))?;
    let selected: Vec<&SpawnGameLevel> = Self::select(&listed, request)?;
    let files: Vec<SpawnLevelFiles> = selected
      .par_iter()
      .map(|level| SpawnLevelFiles::read(&Self::get_level_directory(&request.gamedata, level), &level.name))
      .collect::<XrfResult<_>>()?;
    let inputs: Vec<SpawnRootLevel<'_>> = files
      .iter()
      .zip(&selected)
      .map(|(files, level)| files.get_input(level))
      .collect();
    let build: SpawnRootBuild = SpawnRootBuilder::build(&inputs, &request.options)?;

    build.spawn.write_to_path::<XRayByteOrder, _>(&destination)?;

    Ok(SpawnRootOutcome {
      path: destination.display().to_string(),
      levels: build
        .spawn
        .graphs
        .levels
        .iter()
        .map(|level| level.name.clone())
        .collect(),
      guid: build.spawn.header.guid,
      graph_guid: build.spawn.graphs.header.guid,
      objects: build.spawn.header.objects_count,
      artefact_points: build.spawn.artefact_spawn.nodes.len() as u32,
      patrols: build.spawn.patrols.patrols.len() as u32,
      report: build.report,
    })
  }

  /// The levels asked for, each listed once, or every listed level whose directory is complete.
  fn select<'a>(listed: &'a SpawnGameLevels, request: &SpawnRootRequest) -> XrfResult<Vec<&'a SpawnGameLevel>> {
    let Some(names) = &request.levels else {
      let complete: Vec<&SpawnGameLevel> = listed
        .levels
        .iter()
        .filter(|level| SpawnLevelFiles::is_complete(&Self::get_level_directory(&request.gamedata, level)))
        .collect();

      return if complete.is_empty() {
        Err(XrfError::new_invalid_error(
          "No level the game lists holds level.ai, level.spawn and level.game",
        ))
      } else {
        Ok(complete)
      };
    };
    let mut selected: Vec<&SpawnGameLevel> = Vec::new();

    for name in names {
      let level: &SpawnGameLevel = listed.find(name)?;

      if selected.iter().any(|it| it.name == level.name) {
        return Err(XrfError::new_invalid_error(format!("Level {name} is asked for twice")));
      }

      selected.push(level);
    }

    Ok(selected)
  }

  fn get_level_directory(gamedata: &Path, level: &SpawnGameLevel) -> PathBuf {
    gamedata.join(Self::LEVELS_DIRECTORY).join(&level.name)
  }
}
