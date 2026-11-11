use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_chunk::XRayByteOrder;
use xrf_editable_level::EditableLevelScene;
use xrf_error::{XrfError, XrfResult};
use xrf_gamemtl::GameMtlFile;
use xrf_output::OutputOptions;
use xrf_rawdata::{LevelBuildExport, LevelBuildExporter, LevelExportedPrevious, LevelSceneExporter};
use xrf_utils::format_path;

use crate::commands::level::build::report::LevelBuildReport;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

/// Builds a Level Editor scene's game files into a level directory, as the editor's Build writes them.
#[derive(Default)]
pub(crate) struct BuildCommand;

impl GenericCommand for BuildCommand {
  fn operation(&self) -> &'static str {
    "build"
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about(
        "Build an SDK Level Editor scene's game files into a level directory, as the editor's Build writes them from \
         the scene's parts: level.game, level.env_mod, level.fog_vol, and level.spawn, level_rs.spawn, \
         level.ps_static, level.snd_static, level.snd_env and level.wallmarks where the scene holds them; and given \
         the objects root and game materials, the build.prj xrLC compiles the geometry and lighting from, level.ltx, \
         and level.hom and level.som where objects occlude",
      )
      .arg(
        Arg::new("path")
          .help("Path to the scene's .level; its parts are read from the directory beside it named after it")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Path of the level directory to write the files into")
          .short('d')
          .long("dest")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("objects")
          .help(
            "Objects root ($objects$) the scene's objects are read from; with --game-materials, the build writes \
             build.prj, level.ltx and the occluders",
          )
          .long("objects")
          .requires("game-materials")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("game-materials")
          .help("Game material library, gamemtl.xr, the objects' surfaces name their game materials from")
          .long("game-materials")
          .requires("objects")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("force")
          .help("Overwrite the files the build writes where the directory already holds them")
          .short('f')
          .long("force")
          .action(ArgAction::SetTrue),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let source: &PathBuf = matches.get_one::<PathBuf>("path").expect("the path is required");
    let destination: &PathBuf = matches.get_one::<PathBuf>("dest").expect("the destination is required");
    let output: OutputOptions = context.get_output().clone();

    let geometry: Option<(&PathBuf, &PathBuf)> = matches
      .get_one::<PathBuf>("objects")
      .zip(matches.get_one::<PathBuf>("game-materials"));
    let report: LevelBuildReport = Self::build(&output, source, destination, geometry, matches.get_flag("force"))?;

    context.set_result(|| &report)?;

    Ok(())
  }
}

impl BuildCommand {
  /// Builds the scene at `source` into `destination`, and what it gives the level compiler where `geometry` gives the
  /// objects root and the game material library.
  fn build(
    output: &OutputOptions,
    source: &Path,
    destination: &Path,
    geometry: Option<(&PathBuf, &PathBuf)>,
    is_forced: bool,
  ) -> XrfResult<LevelBuildReport> {
    let scene: EditableLevelScene = EditableLevelScene::read_from_path::<XRayByteOrder, _>(&source)?;
    // The geometry first: the entities it cuts out open the spawn list. A stale file of its own left beside the ones
    // it writes is refused alike.
    let mut names: Vec<&'static str> = LevelSceneExporter::FILE_NAMES.to_vec();
    let (mut files, census, entities, refused) = match geometry {
      Some((objects, game_materials)) => {
        let mut export: LevelBuildExport = LevelBuildExporter::export::<XRayByteOrder>(
          &scene,
          objects,
          &GameMtlFile::read_from_path::<XRayByteOrder, _>(game_materials)?,
        )?;
        let (census, entities, refused) = (
          export.census,
          std::mem::take(&mut export.entities),
          std::mem::take(&mut export.refused),
        );

        names.extend(LevelBuildExport::FILE_NAMES);

        (export.into_files::<XRayByteOrder>()?, Some(census), entities, refused)
      }
      None => (BTreeMap::new(), None, Vec::new(), Vec::new()),
    };

    files.extend(LevelSceneExporter::export::<XRayByteOrder>(&scene, &entities)?);

    for refusal in &refused {
      xrf_output::warning!(output, "{}: {}", refusal.object, refusal.reason);
    }

    if !is_forced && let Some(existing) = files.keys().find(|name| destination.join(name).exists()) {
      return Err(XrfError::new_invalid_error(format!(
        "{} already exists; pass --force to overwrite the files the build writes",
        format_path(&destination.join(existing))
      )));
    }

    let report: LevelBuildReport = LevelBuildReport {
      source: format_path(source).to_string(),
      destination: format_path(destination).to_string(),
      files: LevelSceneExporter::write_to_directory(&files, &names, destination)?,
      geometry: census,
    };

    xrf_output::success!(
      output,
      "Built {} into {}: {}",
      report.source,
      report.destination,
      report
        .files
        .iter()
        .map(|file| format!(
          "{} ({})",
          file.name,
          match file.previous {
            LevelExportedPrevious::New => "new",
            LevelExportedPrevious::Same => "same",
            LevelExportedPrevious::Changed => "changed",
          }
        ))
        .collect::<Vec<_>>()
        .join(", ")
    );

    Ok(report)
  }
}
