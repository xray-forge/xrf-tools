use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_job::{JobHandle, JobScope};

use crate::gamedata::spawn_game_level::SpawnGameLevel;
use crate::gamedata::spawn_game_levels::SpawnGameLevels;
use crate::gamedata::spawn_level_files::SpawnLevelFiles;
use crate::gamedata::spawn_listed_level::SpawnListedLevel;
use crate::gamedata::spawn_root_outcome::SpawnRootOutcome;
use crate::gamedata::spawn_root_request::SpawnRootRequest;
use crate::root::spawn_root_build::SpawnRootBuild;
use crate::root::spawn_root_builder::SpawnRootBuilder;
use crate::root::spawn_root_level::SpawnRootLevel;

/// The root spawn build over a gamedata root, as `xrAI -s` runs over `$game_levels$` and writes `$game_spawn$`: the
/// levels the game lists read, each level's graph built (in parallel; the result is the same), the spawn built and
/// written.
pub struct SpawnRootGamedata;

impl SpawnRootGamedata {
  /// The game's own spawn, which a build writes over only when forced.
  pub(crate) const GAME_SPAWN: &'static str = "all";
  /// What a job reports a build under: its stages (the levels read, the spawn built, the spawn written), each named as
  /// its detail.
  pub const PROGRESS_STAGES: &'static str = "stages";

  /// Builds the spawn `request` asks for and writes it, stopping between its stages where `job` is cancelled.
  ///
  /// # Errors
  ///
  /// Returns an error when the spawn's name is no plain file name, the request names an empty list of levels, the
  /// level list does not read, names no level to build or not one asked for, a level's files do not read, the build
  /// refuses ([`SpawnRootBuilder::build`]), the game's own spawn is there and the request is not forced, the job is
  /// cancelled before writing, or the spawn cannot be written.
  pub fn build(request: &SpawnRootRequest, job: &JobHandle) -> XrfResult<SpawnRootOutcome> {
    if request.name.is_empty() || Path::new(&request.name).file_name() != Some(OsStr::new(&request.name)) {
      return Err(XrfError::new_invalid_error(format!(
        "The spawn's name '{}' is not a file name",
        request.name
      )));
    }

    let destination: PathBuf = request.get_spawn_path();

    if request.name == Self::GAME_SPAWN && destination.exists() && !request.is_forced {
      return Err(XrfError::new_invalid_error(format!(
        "{} is the game's own spawn; force the build to write over it",
        destination.display()
      )));
    }

    let stages: JobScope = job.enter(Self::PROGRESS_STAGES, Some(3));

    job.set_detail(Some(String::from("Reading the levels")));

    let levels_root: PathBuf = request.get_levels_root();
    let listed: SpawnGameLevels = SpawnGameLevels::read_from_path(&request.gamedata.join(SpawnGameLevels::PATH))?;
    let selected: Vec<&SpawnGameLevel> = Self::select(&listed, request, &levels_root)?;
    let files: Vec<SpawnLevelFiles> = selected
      .par_iter()
      .map(|level| SpawnLevelFiles::read(&levels_root.join(&level.name), &level.name))
      .collect::<XrfResult<_>>()?;
    let inputs: Vec<SpawnRootLevel<'_>> = files
      .iter()
      .zip(&selected)
      .map(|(files, level)| files.get_input(level))
      .collect();

    stages.advance();
    job.check_cancelled()?;
    job.set_detail(Some(String::from("Building the spawn")));

    let build: SpawnRootBuild = SpawnRootBuilder::build(&inputs, &request.options)?;

    stages.advance();
    job.check_cancelled()?;
    job.set_detail(Some(destination.display().to_string()));
    build.spawn.write_to_path::<XRayByteOrder, _>(&destination)?;
    stages.advance();

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

  /// The levels `gamedata`'s `configs/game_levels.ltx` lists, in its order, each marked by whether its directory under
  /// `levels_root` holds what a build reads; the entries the build passes over are left out.
  ///
  /// # Errors
  ///
  /// Returns an error naming the file when the level list does not read.
  pub fn list_levels(gamedata: &Path, levels_root: &Path) -> XrfResult<Vec<SpawnListedLevel>> {
    let listed: SpawnGameLevels = SpawnGameLevels::read_from_path(&gamedata.join(SpawnGameLevels::PATH))?;

    Ok(
      listed
        .levels
        .into_iter()
        .map(|level| SpawnListedLevel {
          is_complete: SpawnLevelFiles::is_complete(&levels_root.join(&level.name)),
          name: level.name,
          caption: level.caption,
          id: level.id,
        })
        .collect(),
    )
  }

  /// The levels asked for, each listed once, or every listed level whose directory under `levels_root` is complete.
  fn select<'a>(
    listed: &'a SpawnGameLevels,
    request: &SpawnRootRequest,
    levels_root: &Path,
  ) -> XrfResult<Vec<&'a SpawnGameLevel>> {
    let Some(names) = &request.levels else {
      let complete: Vec<&SpawnGameLevel> = listed
        .levels
        .iter()
        .filter(|level| SpawnLevelFiles::is_complete(&levels_root.join(&level.name)))
        .collect();

      return if complete.is_empty() {
        Err(XrfError::new_invalid_error(
          "No level the game lists holds level.ai, level.spawn and level.game",
        ))
      } else {
        Ok(complete)
      };
    };

    if names.is_empty() {
      return Err(XrfError::new_invalid_error("No level is asked for"));
    }

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
}
