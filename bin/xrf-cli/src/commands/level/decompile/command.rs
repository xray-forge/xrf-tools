use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_build_info::{BuildInfo, build_info};
use xrf_chunk::XRayByteOrder;
use xrf_editable_level::{EditableLevelFile, EditableLevelPart};
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;
use xrf_rawdata::{LevelDecompileOptions, LevelSceneDecompile, LevelSceneDecompiler, LevelSceneProvenance};
use xrf_utils::format_path;

use crate::commands::level::decompile::report::{LevelDecompileReport, LevelDecompileTool};
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
         sounds, sound environments, fog volumes, wall marks, patrol paths, respawn points and environment modifiers) \
         and the level's glows; no geometry yet",
      )
      .arg(
        Arg::new("path")
          .help("Path to the level's directory, holding level, level.game, level.ps_static and the rest")
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
        Arg::new("force")
          .help("Overwrite an existing .level, its parts and its sidecar")
          .short('f')
          .long("force")
          .action(ArgAction::SetTrue),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let source: &PathBuf = matches.get_one::<PathBuf>("path").expect("the path is required");
    let destination: &PathBuf = matches.get_one::<PathBuf>("dest").expect("the destination is required");
    let output: OutputOptions = context.get_output().clone();

    let report: LevelDecompileReport = Self::decompile(&output, source, destination, matches.get_flag("force"))?;

    context.set_result(|| &report)?;

    Ok(())
  }
}

impl DecompileCommand {
  fn decompile(
    output: &OutputOptions,
    source: &Path,
    destination: &Path,
    is_forced: bool,
  ) -> XrfResult<LevelDecompileReport> {
    let name: String = Self::get_level_name(destination)?;
    let sidecar: PathBuf = LevelSceneProvenance::sidecar_path(destination);

    if !is_forced && let Some(existing) = LevelSceneDecompile::get_existing_paths(destination).first() {
      return Err(XrfError::new_invalid_error(format!(
        "{} already exists; pass --force to overwrite the scene",
        format_path(existing)
      )));
    }

    let build: BuildInfo = build_info!();
    let decompiled: LevelSceneDecompile = LevelSceneDecompiler::decompile::<XRayByteOrder>(
      source,
      &LevelDecompileOptions {
        name,
        tool: String::from("xrf-cli level decompile"),
        tool_version: match build.short_commit() {
          Some(commit) => format!("{} ({commit})", build.version),
          None => build.version.to_string(),
        },
      },
    )?;

    decompiled.write_to_path::<XRayByteOrder>(destination)?;

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
    };

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
