use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_chunk::XRayByteOrder;
use xrf_output::OutputOptions;
use xrf_rawdata::{
  LevelBuildExporter, LevelBuildImpostorLimit, LevelBuildReport, LevelBuildRequest, LevelExportedPrevious,
};

use crate::core::arguments::InputArgument;
use crate::core::command::{CommandContext, CommandInput, CommandResult, GenericCommand};
use crate::core::terminal::CliJob;

/// Builds a Level Editor scene's game files into a level directory, as the editor's Build writes them.
#[derive(Default)]
pub(crate) struct BuildCommand;

impl GenericCommand for BuildCommand {
  fn operation(&self) -> &'static str {
    "build"
  }

  fn input(&self) -> CommandInput {
    CommandInput::path("Path to the scene's .level; its parts are read from the directory beside it named after it")
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about(
        "Build an SDK Level Editor scene into a level directory, as the editor's Build writes it. From the scene's \
         parts it writes level.game, level.env_mod and level.fog_vol, and level.spawn, level_rs.spawn, \
         level.ps_static, level.snd_static, level.snd_env and level.wallmarks where the scene holds them. Given \
         --game-materials it also writes what level compile reads, from the scene's objects: build.prj with the scene's build \
         options, level.ltx, level.hom and level.som where objects occlude, build.details and its atlas \
         build_details.dds where the scene holds details, build.aimap where it holds an AI map, the impostor atlas \
         level_lods.dds and level_lods_nm.dds where a multi-use object has an impostor, and each breakable part as \
         meshes/brkbl#<n>.ogf",
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
            "Objects root ($objects$) the scene's objects are read from, repeatable, each object from the first root \
             holding it; by default the objects directory beside the scene's levels directory, where level \
             decompile writes them",
          )
          .long("objects")
          .action(ArgAction::Append)
          .requires("game-materials")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("textures")
          .help(
            "Textures root ($game_textures$) the detail objects' textures are merged from into build_details.dds, and \
             the multi-use objects' impostor strips (lod/lod_<object>.dds and _nm) into level_lods.dds and \
             level_lods_nm.dds; by default gamedata/textures beside the scene's rawdata directory, where level \
             decompile writes them",
          )
          .long("textures")
          .requires("game-materials")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("game-materials")
          .help(
            "Game material library, gamemtl.xr, the objects' surfaces name their game materials from; with it, the \
             build writes build.prj, level.ltx and the occluders",
          )
          .long("game-materials")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("sdk-impostor-atlas")
          .help(
            "Keep the impostor atlas within SDK 0.7's 2048 by 2048, refusing a level of more than 128 impostors as that \
             Level Editor does; by default it grows to 4096 a side, as Anomaly's largest levels ship it",
          )
          .long("sdk-impostor-atlas")
          .requires("game-materials")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("force")
          .help(
            "Overwrite the files the build writes where the directory already holds them, and remove those of its own \
             an earlier build wrote that this one does not; without it, a directory holding one is refused before the \
             scene is read",
          )
          .short('f')
          .long("force")
          .action(ArgAction::SetTrue),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let request: LevelBuildRequest = LevelBuildRequest {
      source: InputArgument::of(matches).clone(),
      destination: matches
        .get_one::<PathBuf>("dest")
        .expect("the destination is required")
        .clone(),
      objects: matches
        .get_many::<PathBuf>("objects")
        .map(|roots| roots.cloned().collect())
        .unwrap_or_default(),
      game_materials: matches.get_one::<PathBuf>("game-materials").cloned(),
      textures: matches.get_one::<PathBuf>("textures").cloned(),
      impostor_limit: if matches.get_flag("sdk-impostor-atlas") {
        LevelBuildImpostorLimit::Sdk
      } else {
        LevelBuildImpostorLimit::Extended
      },
      is_forced: matches.get_flag("force"),
    };
    let report: LevelBuildReport =
      LevelBuildExporter::build_to_directory::<XRayByteOrder>(&request, &CliJob::for_output(context.get_output()))?;

    Self::print(context.get_output(), &report);
    context.set_result(|| &report)?;

    Ok(())
  }
}

impl BuildCommand {
  /// What the build wrote, each file with what it replaced, what of its own it removed, and each part it refused.
  fn print(output: &OutputOptions, report: &LevelBuildReport) {
    for refusal in &report.refused {
      xrf_output::warning!(output, "{}: {}", refusal.object, refusal.reason);
    }

    if !report.removed.is_empty() {
      xrf_output::info!(
        output,
        "Removed what an earlier build wrote and this one does not: {}",
        report.removed.join(", ")
      );
    }

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
  }
}
