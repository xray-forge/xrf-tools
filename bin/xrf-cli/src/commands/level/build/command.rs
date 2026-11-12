use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_chunk::XRayByteOrder;
use xrf_output::OutputOptions;
use xrf_rawdata::{
  LevelBuildExporter, LevelBuildImpostorLimit, LevelBuildReport, LevelBuildRequest, LevelExportedPrevious,
};

use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::terminal::CliJob;

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
         the objects roots and game materials, the build.prj xrLC compiles the geometry and lighting from, level.ltx, \
         level.hom and level.som where objects occlude, build.details and its atlas build_details.dds where the \
         scene holds details, and the impostor atlas level_lods.dds and level_lods_nm.dds where a multi-use object \
         has an impostor, and each breakable part as meshes/brkbl#<n>.ogf",
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
            "Objects root ($objects$) the scene's objects are read from, repeatable, each object from the first root \
             holding it; with --game-materials, the build writes build.prj, level.ltx and the occluders",
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
             level_lods_nm.dds; a scene of details or impostors needs it",
          )
          .long("textures")
          .requires("objects")
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
        Arg::new("proof-lighting")
          .help(
            "Light build.prj at proof quality whatever the scene sets, 3 lightmap pixels a metre, 1 jitter sample, sky \
             and sun at quality 1, to check a level with xrLC in minutes rather than hours; not for a level to ship",
          )
          .long("proof-lighting")
          .requires("objects")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("sdk-impostor-atlas")
          .help(
            "Keep the impostor atlas within SDK 0.7's 2048 by 2048, refusing a level of more than 128 impostors as that \
             Level Editor does; by default it grows to 4096 a side, as Anomaly's largest levels ship it",
          )
          .long("sdk-impostor-atlas")
          .requires("objects")
          .action(ArgAction::SetTrue),
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
    let request: LevelBuildRequest = LevelBuildRequest {
      source: matches
        .get_one::<PathBuf>("path")
        .expect("the path is required")
        .clone(),
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
      is_proof_lighting: matches.get_flag("proof-lighting"),
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
  /// What the build wrote, each file with what it replaced, and each part it refused.
  fn print(output: &OutputOptions, report: &LevelBuildReport) {
    for refusal in &report.refused {
      xrf_output::warning!(output, "{}: {}", refusal.object, refusal.reason);
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
