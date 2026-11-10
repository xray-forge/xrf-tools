use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_build_info::{BuildInfo, build_info};
use xrf_chunk::XRayByteOrder;
use xrf_editable_level::{EditableLevelFile, EditableLevelPart};
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;
use xrf_rawdata::{
  LevelDecompileOptions, LevelSceneDecompile, LevelSceneDecompiler, LevelSceneProvenance, LevelSurfaceInference,
  LevelSurfaces,
};
use xrf_utils::format_path;

use crate::commands::level::decompile::report::{
  LevelDecompileGeometry, LevelDecompileReport, LevelDecompileRootSpawn, LevelDecompileSurfaces, LevelDecompileTool,
};
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

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
            "Object library root, such as an SDK's rawdata/objects, whose surfaces' pairings of engine and compiler \
             shaders break ties between compiler shaders that fit; repeatable",
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
    let source: &PathBuf = matches.get_one::<PathBuf>("path").expect("the path is required");
    let destination: &PathBuf = matches.get_one::<PathBuf>("dest").expect("the destination is required");
    let output: OutputOptions = context.get_output().clone();

    let gamedata: PathBuf = Self::get_gamedata(source);
    let surfaces: LevelSurfaces = match matches.get_one::<String>("surfaces").map(String::as_str) {
      Some("defaults") => LevelSurfaces::Defaults,
      _ => LevelSurfaces::Infer,
    };
    // A library named must exist; one by default is taken where the level's gamedata holds it.
    let library = |name: &str, file: &str| -> XrfResult<Option<PathBuf>> {
      match matches.get_one::<PathBuf>(name) {
        Some(path) if path.is_file() => Ok(Some(path.clone())),
        Some(path) => Err(XrfError::new_not_found_error(format!(
          "{} does not exist",
          format_path(path)
        ))),
        None => Ok(Some(gamedata.join(file)).filter(|path| path.is_file())),
      }
    };
    let build: BuildInfo = build_info!();
    let options: LevelDecompileOptions = LevelDecompileOptions {
      name: Self::get_level_name(destination)?,
      tool: String::from("xrf-cli level decompile"),
      tool_version: match build.short_commit() {
        Some(commit) => format!("{} ({commit})", build.version),
        None => build.version.to_string(),
      },
      root_spawn: Self::get_root_spawn(&gamedata, matches.get_one::<String>("spawn").map(String::as_str))?,
      surfaces,
      game_materials: library("game-materials", LevelSurfaceInference::GAME_MATERIALS_FILE)?,
      compiler_shaders: library("compiler-shaders", LevelSurfaceInference::COMPILER_SHADERS_FILE)?,
      libraries: matches
        .get_many::<PathBuf>("library")
        .map(|roots| roots.cloned().collect())
        .unwrap_or_default(),
    };
    let objects: PathBuf = matches
      .get_one::<PathBuf>("objects")
      .cloned()
      .unwrap_or_else(|| Self::get_default_objects_root(destination));
    let report: LevelDecompileReport = Self::decompile(
      &output,
      source,
      destination,
      &objects,
      &options,
      matches.get_flag("force"),
    )?;

    context.set_result(|| &report)?;

    Ok(())
  }
}

impl DecompileCommand {
  /// The root spawn a new game starts from, which a decompile reads by default (decision 10).
  const DEFAULT_ROOT_SPAWN: &'static str = "all";

  /// The `gamedata` directory holding `levels\<level>`, named by the level path's parents where it has them.
  fn get_gamedata(source: &Path) -> PathBuf {
    match source.parent().and_then(Path::parent) {
      Some(gamedata) if !gamedata.as_os_str().is_empty() => gamedata.to_path_buf(),
      _ => source.join("..").join(".."),
    }
  }

  /// The root spawn named `name` in `gamedata`, `spawns/<name>.spawn`. Without a name, `all.spawn` where it exists,
  /// else none.
  fn get_root_spawn(gamedata: &Path, name: Option<&str>) -> XrfResult<Option<PathBuf>> {
    let path: PathBuf = gamedata
      .join("spawns")
      .join(format!("{}.spawn", name.unwrap_or(Self::DEFAULT_ROOT_SPAWN)));

    match (path.is_file(), name) {
      (true, _) => Ok(Some(path)),
      (false, None) => Ok(None),
      (false, Some(_)) => Err(XrfError::new_not_found_error(format!(
        "{} does not exist",
        format_path(&path)
      ))),
    }
  }

  /// The objects root beside the scene's directory, `<levels>/../objects`, as the SDK's `fs.ltx` puts `$maps$` and
  /// `$objects$` side by side under its rawdata.
  pub(crate) fn get_default_objects_root(destination: &Path) -> PathBuf {
    let named = |path: &&Path| !path.as_os_str().is_empty();

    match destination.parent().filter(named) {
      Some(levels) => match levels.parent().filter(named) {
        Some(rawdata) => rawdata.join("objects"),
        None => levels.join("..").join("objects"),
      },
      None => Path::new("..").join("objects"),
    }
  }

  fn decompile(
    output: &OutputOptions,
    source: &Path,
    destination: &Path,
    objects: &Path,
    options: &LevelDecompileOptions,
    is_forced: bool,
  ) -> XrfResult<LevelDecompileReport> {
    let sidecar: PathBuf = LevelSceneProvenance::sidecar_path(destination);

    if !is_forced && let Some(existing) = LevelSceneDecompile::get_existing_paths(destination).first() {
      return Err(XrfError::new_invalid_error(format!(
        "{} already exists; pass --force to overwrite the scene",
        format_path(existing)
      )));
    }

    let decompiled: LevelSceneDecompile = LevelSceneDecompiler::decompile::<XRayByteOrder>(source, options)?;

    if !is_forced && let Some(existing) = decompiled.get_existing_object_paths(objects).first() {
      return Err(XrfError::new_invalid_error(format!(
        "{} already exists; pass --force to overwrite the scene's objects",
        format_path(existing)
      )));
    }

    decompiled.write_to_path::<XRayByteOrder>(destination, objects)?;

    let report: LevelDecompileReport = LevelDecompileReport {
      source: format_path(source).to_string(),
      destination: format_path(destination).to_string(),
      sidecar: format_path(&sidecar).to_string(),
      sources: decompiled
        .provenance
        .sources
        .iter()
        .map(|source| source.file.clone())
        .collect(),
      tools: decompiled
        .scene
        .tools
        .iter()
        .map(|tool| {
          Ok(LevelDecompileTool {
            name: tool.name.clone(),
            objects: match tool.parts.first() {
              Some(EditableLevelPart::Text(part)) => Some(part.get_objects()?.len()),
              _ => None,
            },
          })
        })
        .collect::<XrfResult<Vec<LevelDecompileTool>>>()?,
      inferred: decompiled.provenance.inferred,
      refused: decompiled.provenance.refused,
      root_spawn: decompiled.provenance.root_spawn.map(|root| LevelDecompileRootSpawn {
        file: root.file,
        objects: root.objects,
        disagreements: root.disagreements.len(),
        update_packets: root.update_packets.len(),
      }),
      objects_root: format_path(objects).to_string(),
      geometry: decompiled.provenance.geometry.map(|geometry| LevelDecompileGeometry {
        objects: geometry.objects.len(),
        census: geometry.census,
        surfaces: geometry.surfaces.map(|surfaces| LevelDecompileSurfaces {
          exact_triangles: surfaces.census.exact_triangles,
          traced_triangles: surfaces.census.traced_triangles,
          uncollided_triangles: surfaces.census.uncollided_triangles,
          compiler_shaders: surfaces.census.compiler_shaders.len(),
          unknown_game_materials: surfaces.census.unknown_game_materials.len(),
        }),
      }),
    };

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
    }

    if let Some(unknown) = report
      .geometry
      .as_ref()
      .and_then(|geometry| geometry.surfaces.as_ref())
      .map(|surfaces| surfaces.unknown_game_materials)
      .filter(|unknown| *unknown > 0)
    {
      xrf_output::warning!(
        output,
        "{unknown} collision materials are not in the game material library; their surfaces take default"
      );
    }

    if let Some(geometry) = &report.geometry {
      xrf_output::info!(
        output,
        "Static geometry: {} objects under {}, {} faces from {} triangles ({} two-sided folds, {} repeats dropped); \
         {} tree triangles left for their own objects; {} collision faces no render triangle draws",
        geometry.objects,
        report.objects_root,
        geometry.census.faces,
        geometry.census.static_triangles,
        geometry.census.two_sided_faces,
        geometry.census.repeated_triangles,
        geometry.census.tree_triangles,
        geometry.census.collision_only_faces
      );
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

    Ok(report)
  }

  /// The level's name, the destination's file stem, from a path ending in `.level`.
  fn get_level_name(destination: &Path) -> XrfResult<String> {
    destination
      .extension()
      .filter(|extension| extension.eq_ignore_ascii_case(EditableLevelFile::EXTENSION))
      .and(destination.file_stem())
      .and_then(|stem| stem.to_str())
      .filter(|stem| !stem.is_empty())
      .map(str::to_owned)
      .ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "{} does not name a .{} file",
          format_path(destination),
          EditableLevelFile::EXTENSION
        ))
      })
  }
}
