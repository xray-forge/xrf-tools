use clap::{ArgMatches, Command};
use xrf_chunk::{ChunkReader, InMemoryChunkDataSource};
use xrf_level::{LevelFile, LevelGeomFile, LevelVisualsChunk};
use xrf_output::OutputOptions;
use xrf_spawn::XRayByteOrder;

use super::level_visuals_report::LevelVisualsReport;
use super::report::LevelInfoReport;
use crate::commands::level::level_assets::LevelAssets;
use crate::commands::level::level_selection::LevelSelection;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

#[derive(Default)]
pub struct InfoCommand;

impl GenericCommand for InfoCommand {
  fn operation(&self) -> &'static str {
    "info"
  }

  fn declare(&self, command: Command) -> Command {
    LevelSelection::declare(command.about("Print what a compiled level holds"))
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let selection: LevelSelection = LevelSelection::of(matches)?;
    let output: OutputOptions = context.get_output().clone();

    // The mount outlives the assets that borrow from it, so it is taken before the level is opened.
    let assets: LevelAssets = selection.open()?;

    xrf_output::info!(output, "Read compiled level {}", assets.describe());

    let bundle: Vec<u8> = assets.read(LevelFile::FILE_NAME)?.ok_or_else(|| {
      xrf_error::XrfError::new_not_found_error(format!(
        "Level bundle was not found: {}",
        assets.describe_file(LevelFile::FILE_NAME)
      ))
    })?;

    // Split once and read twice: an archived bundle is the largest file a level ships.
    let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_vec(bundle)?.read_children()?;
    let level: LevelFile = LevelFile::read_from_chunks::<XRayByteOrder, _>(&chunks)?;
    let visuals: Option<LevelVisualsChunk> = LevelFile::read_visuals_from_chunks::<XRayByteOrder, _>(&chunks)?;

    let geometry: Option<LevelGeomFile> = Self::read_geometry(&assets, LevelGeomFile::FILE_NAME, &output)?;
    let detail_geometry: Option<LevelGeomFile> =
      Self::read_geometry(&assets, LevelGeomFile::DETAIL_FILE_NAME, &output)?;

    xrf_output::info!(
      output,
      "Built by xrLC version {}, quality {}",
      level.header.xrlc_version,
      level.header.xrlc_quality
    );

    xrf_output::info!(
      output,
      "{} sectors, {} portals, {} static lights{}",
      level.sectors.as_ref().map_or(0, |it| it.sectors.len()),
      level.portals.as_ref().map_or(0, |it| it.portals.len()),
      level.lights.as_ref().map_or(0, |it| it.lights.len()),
      match level.lights.as_ref().and_then(|it| it.get_sun()) {
        Some(_) => ", one of them the sun",
        None => "",
      }
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

    for (label, file) in [
      (LevelGeomFile::FILE_NAME, &geometry),
      (LevelGeomFile::DETAIL_FILE_NAME, &detail_geometry),
    ] {
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
  fn read_geometry(assets: &LevelAssets, name: &str, output: &OutputOptions) -> CommandResult<Option<LevelGeomFile>> {
    let Some(bytes) = assets.read(name)? else {
      xrf_output::verbose!(output, "No {} beside the level", name);

      return Ok(None);
    };

    Ok(Some(LevelGeomFile::read_from_bytes::<XRayByteOrder>(bytes)?))
  }
}
