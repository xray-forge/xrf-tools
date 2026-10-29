use clap::{ArgMatches, Command};
use xrf_chunk::{ChunkReader, InMemoryChunkDataSource, XRayByteOrder};
use xrf_error::XrfError;
use xrf_gamemtl::GameMtlFile;
use xrf_level::{
  LevelCformFile, LevelCformGeometry, LevelCformTracer, LevelFile, LevelGeomFile, LevelGeomSource,
  LevelSurfaceMaterials, LevelVisualsChunk,
};
use xrf_output::OutputOptions;

use super::report::LevelMaterialsReport;
use crate::commands::level::level_assets::LevelAssets;
use crate::commands::level::level_selection::LevelSelection;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

#[derive(Default)]
pub struct MaterialsCommand;

impl GenericCommand for MaterialsCommand {
  fn operation(&self) -> &'static str {
    "materials"
  }

  fn declare(&self, command: Command) -> Command {
    LevelSelection::declare(
      command.about("Report the game materials each of a level's render surfaces lies on, read off its collision form"),
    )
  }

  /// Tallies every drawable visual's triangles against the collision faces beneath them, per shader table entry.
  ///
  /// The materials are named from the roots' `gamemtl.xr`; a level directory read alone reports their ids.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let selection: LevelSelection = LevelSelection::of(matches)?;
    let output: OutputOptions = context.get_output().clone();
    let assets: LevelAssets = selection.open()?;
    let require = |name: &str| {
      assets.read(name)?.ok_or_else(|| {
        XrfError::new_not_found_error(format!("Level file was not found: {}", assets.describe_file(name)))
      })
    };

    xrf_output::info!(
      output,
      "Reading the materials beneath compiled level {}",
      assets.describe()
    );

    // Split once and read twice: an archived bundle is the largest file a level ships.
    let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> =
      ChunkReader::from_vec(require(LevelFile::FILE_NAME)?)?.read_children()?;
    let level: LevelFile = LevelFile::read_from_chunks::<XRayByteOrder, _>(&chunks)?;
    let visuals: LevelVisualsChunk =
      LevelFile::read_visuals_from_chunks::<XRayByteOrder, _>(&chunks)?.ok_or_else(|| {
        XrfError::new_not_found_error("The level carries no visuals, so it draws nothing to look under")
      })?;
    let source: LevelGeomSource<InMemoryChunkDataSource> =
      LevelGeomSource::open_from_bytes::<XRayByteOrder>(require(LevelGeomFile::FILE_NAME)?)?;
    let collision: LevelCformGeometry =
      LevelCformFile::read_geometry_from_bytes::<XRayByteOrder>(require(LevelCformFile::FILE_NAME)?)?;
    let library: Option<GameMtlFile> = Self::read_library(&assets, &output);

    let survey: LevelSurfaceMaterials =
      LevelSurfaceMaterials::survey::<XRayByteOrder, _>(&visuals, &source, &LevelCformTracer::new(&collision));
    let report: LevelMaterialsReport =
      LevelMaterialsReport::new(assets.describe(), &survey, level.shaders.as_ref(), library.as_ref());

    Self::print(&output, &report);

    context.set_result(|| &report)?;

    Ok(())
  }
}

impl MaterialsCommand {
  /// The game material library of the roots the level was read from, to name the ids by; none for a level directory,
  /// or where it cannot be read, which leaves the ids unnamed rather than failing the report.
  fn read_library(assets: &LevelAssets, output: &OutputOptions) -> Option<GameMtlFile> {
    let LevelAssets::Asset { search, .. } = assets else {
      return None;
    };

    match search
      .read(GameMtlFile::FILE_NAME)
      .and_then(GameMtlFile::read_from_bytes::<XRayByteOrder>)
    {
      Ok(library) => Some(library),
      Err(error) => {
        xrf_output::warning!(
          output,
          "Materials are left unnamed, {} was not read: {error}",
          GameMtlFile::FILE_NAME
        );

        None
      }
    }
  }

  fn print(output: &OutputOptions, report: &LevelMaterialsReport) {
    xrf_output::info!(
      output,
      "{} surfaces over {:.0} square metres, {:.0}% of it on collision faces of {} materials",
      report.summary.surfaces,
      report.summary.area,
      report.summary.covered_share * 100.0,
      report.summary.materials
    );

    if report.summary.unread_visuals > 0 {
      xrf_output::warning!(output, "{} visuals could not be read", report.summary.unread_visuals);
    }

    for surface in &report.surfaces {
      let texture: &str = surface.textures.first().map_or("", String::as_str);

      match surface.materials.first() {
        Some(dominant) => xrf_output::verbose!(
          output,
          "{} {}: {} {:.0}% of {:.0} m2, {:.0}% uncovered",
          surface.shader.as_deref().unwrap_or("?"),
          texture,
          dominant
            .name
            .clone()
            .unwrap_or_else(|| format!("material {}", dominant.id)),
          dominant.share * 100.0,
          surface.area,
          surface.uncovered_share * 100.0
        ),
        None => xrf_output::verbose!(
          output,
          "{} {}: {:.0} m2 on no collision face",
          surface.shader.as_deref().unwrap_or("?"),
          texture,
          surface.area
        ),
      }
    }
  }
}
