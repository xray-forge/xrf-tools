use std::path::{Path, PathBuf};

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_db::{LevelFile, LevelGeomFile, LevelVisualsChunk, XRayByteOrder};
use xrf_output::OutputOptions;
use xrf_utils::format_path;

use super::report::{LevelInfoReport, LevelVisualsReport};
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct InfoCommand;

impl InfoCommand {
  /// The render bundle, which every compiled level has.
  const LEVEL_FILE: &'static str = "level";

  /// Render geometry, which a level built by a compiler that emitted none may lack.
  const GEOMETRY_FILE: &'static str = "level.geom";

  /// The fast-path twin of the geometry, absent for a level with no fast geometry.
  const DETAIL_GEOMETRY_FILE: &'static str = "level.geomx";
}

impl GenericCommand for InfoCommand {
  fn operation(&self) -> &'static str {
    "info"
  }

  /// Create command for printing compiled level info.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to print information about a compiled level directory")
      .arg(
        Arg::new("path")
          .help("Path to a compiled level directory, the one holding `level` and `level.geom`")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
  }

  /// Print information about a compiled level.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<_>("path")
      .expect("Expected valid path to be provided");

    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(output, "Read compiled level {}", format_path(path));

    let level: LevelFile = LevelFile::read_from_path::<XRayByteOrder, _>(&path.join(Self::LEVEL_FILE))?;
    let visuals: Option<LevelVisualsChunk> =
      LevelFile::read_visuals_from_path::<XRayByteOrder, _>(&path.join(Self::LEVEL_FILE))?;

    let geometry: Option<LevelGeomFile> = Self::read_geometry(path, Self::GEOMETRY_FILE, &output)?;
    let detail_geometry: Option<LevelGeomFile> = Self::read_geometry(path, Self::DETAIL_GEOMETRY_FILE, &output)?;

    xrf_output::info!(
      output,
      "Built by xrLC version {}, quality {}",
      level.header.xrlc_version,
      level.header.xrlc_quality
    );

    match &level.shaders {
      Some(shaders) => xrf_output::info!(output, "Shader table entries: {}", shaders.entries.len()),
      // The renderer asserts on this, so it is worth saying loudly rather than reporting a zero.
      None => xrf_output::warning!(output, "Shader table is absent, the level is not built correctly"),
    }

    match &visuals {
      Some(visuals) => {
        xrf_output::info!(
          output,
          "Visuals: {}, drawable: {}, with fast path: {}",
          visuals.visuals.len(),
          visuals.count_drawable(),
          visuals.count_fastpath()
        );

        for (index, visual) in visuals.visuals.iter().enumerate() {
          xrf_output::verbose!(output, "{}", LevelVisualsReport::describe_visual(index, visual));
        }
      }
      None => xrf_output::warning!(output, "Visuals chunk is absent, the level draws nothing"),
    }

    for (label, file) in [("level.geom", &geometry), ("level.geomx", &detail_geometry)] {
      let Some(file) = file else {
        continue;
      };

      xrf_output::info!(
        output,
        "{}: {} vertex buffers, {} index buffers, {} slide windows",
        label,
        file.vertex_buffers.len(),
        file.index_buffers.len(),
        file.slide_windows.len()
      );
    }

    context
      .set_result(|| LevelInfoReport::new(&level, visuals.as_ref(), geometry.as_ref(), detail_geometry.as_ref()))?;

    Ok(())
  }
}

impl InfoCommand {
  /// Read one render geometry file of a level, or `None` when the level ships without it.
  fn read_geometry(path: &Path, name: &str, output: &OutputOptions) -> CommandResult<Option<LevelGeomFile>> {
    let path: PathBuf = path.join(name);

    if !path.is_file() {
      xrf_output::verbose!(output, "No {} beside the level", name);

      return Ok(None);
    }

    Ok(Some(LevelGeomFile::read_from_path::<XRayByteOrder, _>(&path)?))
  }
}
