use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_chunk::XRayByteOrder;
use xrf_editable_level::EditableLevelScene;
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;
use xrf_rawdata::{LevelExportedPrevious, LevelSceneExporter};
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
         level.ps_static, level.snd_static, level.snd_env and level.wallmarks where the scene holds them. No geometry \
         or build.prj yet",
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

    let report: LevelBuildReport = Self::build(&output, source, destination, matches.get_flag("force"))?;

    context.set_result(|| &report)?;

    Ok(())
  }
}

impl BuildCommand {
  fn build(output: &OutputOptions, source: &Path, destination: &Path, is_forced: bool) -> XrfResult<LevelBuildReport> {
    let scene: EditableLevelScene = EditableLevelScene::read_from_path::<XRayByteOrder, _>(&source)?;
    let files: BTreeMap<&'static str, Vec<u8>> = LevelSceneExporter::export::<XRayByteOrder>(&scene)?;

    if !is_forced && let Some(existing) = files.keys().find(|name| destination.join(name).exists()) {
      return Err(XrfError::new_invalid_error(format!(
        "{} already exists; pass --force to overwrite the files the build writes",
        format_path(&destination.join(existing))
      )));
    }

    let report: LevelBuildReport = LevelBuildReport {
      source: format_path(source).to_string(),
      destination: format_path(destination).to_string(),
      files: LevelSceneExporter::write_to_directory(&files, destination)?,
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
