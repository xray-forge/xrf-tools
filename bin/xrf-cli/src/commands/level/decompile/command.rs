use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_build_info::{BuildInfo, build_info};
use xrf_chunk::XRayByteOrder;
use xrf_output::OutputOptions;
use xrf_rawdata::{LevelDecompileReport, LevelDecompileRequest, LevelSceneDecompiler, LevelSurfaces};

use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::terminal::CliJob;

/// Decompiles a compiled level's directory into a Level Editor scene, with a provenance sidecar beside it.
#[derive(Default)]
pub(crate) struct DecompileCommand;

impl GenericCommand for DecompileCommand {
  fn operation(&self) -> &'static str {
    "decompile"
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about(
        "Decompile a compiled level's directory into an SDK Level Editor scene (.level and its parts), with a \
         .level.xrf.json provenance sidecar beside it. The scene holds the parts the build passes through (particles, \
         sounds, sound environments, fog volumes, wall marks, patrol paths, respawn points, environment modifiers and \
         entity spawns), the level's glows and lights, and its static geometry as one .object per sector with the \
         sectors and portals",
      )
      .arg(
        Arg::new("path")
          .help("Path to the level's directory, holding level, build.lights, level.game, level.ps_static and the rest")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help(
            "Path of the .level to write; its parts go in the directory beside it named after it, its sidecar beside \
             it, and its name is the level's",
          )
          .short('d')
          .long("dest")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("objects")
          .help(
            "Objects root ($objects$) the scene's .object files are written under, at levels/<level>/; by default the \
             objects directory beside the destination's directory, as the SDK lays out rawdata/levels and \
             rawdata/objects",
          )
          .long("objects")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("textures")
          .help(
            "Game textures root ($game_textures$) the textures the objects need for the build, such as the tree \
             models' impostor strips under lod/, are written under; by default gamedata/textures beside the \
             destination's rawdata directory, as the SDK lays out editors/rawdata and editors/gamedata",
          )
          .long("textures")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("surfaces")
          .help(
            "How the geometry's surfaces are dressed: infer each game material from the collision face beneath and a \
             compiler shader fitting what the level shows, or give every surface the editor's defaults",
          )
          .long("surfaces")
          .value_parser(["infer", "defaults"])
          .default_value("infer"),
      )
      .arg(
        Arg::new("game-materials")
          .help("Game material library naming the collision materials; gamemtl.xr in the level's gamedata by default")
          .long("game-materials")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("compiler-shaders")
          .help(
            "Compiler shader library inferred compiler shaders are chosen from; shaders_xrlc.xr in the level's gamedata \
             by default",
          )
          .long("compiler-shaders")
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("library")
          .help(
            "Object library root, such as an SDK's rawdata/objects: its objects found placed in the static geometry and \
             its trees found as tree models are placed by reference and copied beside the objects, and its surfaces' \
             pairings of engine and compiler shaders break ties between compiler shaders that fit; repeatable",
          )
          .long("library")
          .action(ArgAction::Append)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("spawn")
          .help(
            "Root spawn whose objects on the level replace level.spawn's entities, read from spawns/<spawn>.spawn \
             beside the level's levels directory; all.spawn by default where it exists",
          )
          .long("spawn")
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("force")
          .help("Overwrite an existing .level, its parts, its sidecar and its objects")
          .short('f')
          .long("force")
          .action(ArgAction::SetTrue),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let build: BuildInfo = build_info!();
    let request: LevelDecompileRequest = LevelDecompileRequest {
      source: matches
        .get_one::<PathBuf>("path")
        .expect("the path is required")
        .clone(),
      destination: matches
        .get_one::<PathBuf>("dest")
        .expect("the destination is required")
        .clone(),
      objects: matches.get_one::<PathBuf>("objects").cloned(),
      textures: matches.get_one::<PathBuf>("textures").cloned(),
      root_spawn: matches.get_one::<String>("spawn").cloned(),
      surfaces: match matches.get_one::<String>("surfaces").map(String::as_str) {
        Some("defaults") => LevelSurfaces::Defaults,
        _ => LevelSurfaces::Infer,
      },
      game_materials: matches.get_one::<PathBuf>("game-materials").cloned(),
      compiler_shaders: matches.get_one::<PathBuf>("compiler-shaders").cloned(),
      libraries: matches
        .get_many::<PathBuf>("library")
        .map(|roots| roots.cloned().collect())
        .unwrap_or_default(),
      is_forced: matches.get_flag("force"),
    };
    let report: LevelDecompileReport = LevelSceneDecompiler::decompile_to_path::<XRayByteOrder>(
      &request,
      "xrf-cli level decompile",
      &match build.short_commit() {
        Some(commit) => format!("{} ({commit})", build.version),
        None => build.version.to_string(),
      },
      &CliJob::for_output(context.get_output()),
    )?;

    Self::print(context.get_output(), &report);
    context.set_result(|| &report)?;

    Ok(())
  }
}

impl DecompileCommand {
  /// What the decompile wrote, a line per kind of it, warnings where it inferred or left something out.
  fn print(output: &OutputOptions, report: &LevelDecompileReport) {
    if let Some(surfaces) = report.geometry.as_ref().and_then(|geometry| geometry.surfaces.as_ref()) {
      xrf_output::info!(
        output,
        "Surfaces: game materials from {} triangles on a collision face and {} traced to one, {} on none; {} compiler \
         shaders chosen, each with its reason in the sidecar",
        surfaces.exact_triangles,
        surfaces.traced_triangles,
        surfaces.uncollided_triangles,
        surfaces.compiler_shaders
      );

      if surfaces.unknown_game_materials > 0 {
        xrf_output::warning!(
          output,
          "{} collision materials are not in the game material library; their surfaces take default",
          surfaces.unknown_game_materials
        );
      }
    }

    if let Some(geometry) = &report.geometry {
      xrf_output::info!(
        output,
        "Static geometry: {} objects under {}, {} faces from {} triangles ({} two-sided folds, {} repeats dropped); \
         {} tree models placed {} times, {} with impostor strips under {}; {} collision faces no render triangle \
         draws",
        geometry.objects,
        report.objects_root,
        geometry.census.faces,
        geometry.census.static_triangles,
        geometry.census.two_sided_faces,
        geometry.census.repeated_triangles,
        geometry.census.tree_models,
        geometry.census.tree_placements,
        geometry.census.tree_impostors,
        report.textures_root,
        geometry.census.collision_only_faces
      );

      if geometry.census.instances + geometry.census.library_tree_models > 0 {
        xrf_output::info!(
          output,
          "Library objects: {} placements of {} objects found in the static geometry, taking {} of its triangles; {} \
           tree models found to be library trees; each copied beside the objects, as the sidecar records",
          geometry.census.instances,
          geometry.census.instance_objects,
          geometry.census.instance_triangles,
          geometry.census.library_tree_models
        );
      }
    }

    if let Some(root) = report.root_spawn.as_ref().filter(|root| root.objects > 0) {
      xrf_output::info!(
        output,
        "Entities from {} ({} objects): {} disagreements with level.spawn and {} update packets recorded in the sidecar",
        root.file,
        root.objects,
        root.disagreements,
        root.update_packets
      );
    }

    if !report.refused.is_empty() {
      xrf_output::warning!(
        output,
        "Left out {} objects the scene cannot hold, each named with its reason in the sidecar",
        report.refused.len()
      );
    }

    xrf_output::success!(
      output,
      "Decompiled {} into {} from {} files: {}; provenance in {}",
      report.source,
      report.destination,
      report.sources.len(),
      report
        .tools
        .iter()
        .map(|tool| match tool.objects {
          Some(objects) => format!("{} {objects}", tool.name),
          None => tool.name.clone(),
        })
        .collect::<Vec<_>>()
        .join(", "),
      report.sidecar
    );
  }
}
