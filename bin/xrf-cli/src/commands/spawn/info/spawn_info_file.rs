use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;

use xrf_chunk::{ChunkReader, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;
use xrf_spawn::{LevelSpawnFile, SpawnFile, SpawnFileKind, XRayByteOrder};
use xrf_utils::format_path;

/// The spawn file `spawn info` read, as whichever of the two `.spawn` formats it is.
pub(crate) enum SpawnInfoFile {
  Root(Box<SpawnFile>),
  Level(LevelSpawnFile),
}

impl SpawnInfoFile {
  /// Reads the file once and tells its kind by its own chunks, since its name cannot.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read or walked as chunks, when a root spawn does not read, or when the
  /// file is neither a root spawn nor a level's spawn list.
  pub(crate) fn read_from_path(path: &Path) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> =
      ChunkReader::from_file(File::open(path)?)?.read_children()?;

    match SpawnFileKind::of_chunks::<XRayByteOrder, _>(&chunks) {
      SpawnFileKind::Root => Ok(Self::Root(Box::new(SpawnFile::read_from_chunks::<XRayByteOrder, _>(
        &chunks,
      )?))),
      SpawnFileKind::Level => LevelSpawnFile::read_from_chunks::<XRayByteOrder, _>(chunks)
        .map(Self::Level)
        .map_err(|error| {
          XrfError::new_invalid_error(format!(
            "Expected a root spawn opening with its header chunk, or a level's spawn list of spawn packets: {} is \
             neither, {error}",
            format_path(path)
          ))
        }),
    }
  }

  /// Prints what the file holds, one line per count.
  pub(crate) fn print(&self, output: &OutputOptions) {
    match self {
      Self::Root(file) => Self::print_root(file, output),
      Self::Level(file) => Self::print_level(file, output),
    }
  }

  fn print_root(file: &SpawnFile, output: &OutputOptions) {
    xrf_output::info!(output, "Spawn file information:");
    xrf_output::info!(output, "Version: {}", file.header.version);
    xrf_output::info!(output, "GUID: {}", file.header.guid);
    xrf_output::info!(output, "Levels count: {}", file.header.levels_count);
    xrf_output::info!(output, "Objects count: {}", file.header.objects_count);
    xrf_output::info!(output, "Artefact spawn points: {}", file.artefact_spawn.nodes.len());
    xrf_output::info!(output, "Patrols: {}", file.patrols.patrols.len());
    xrf_output::info!(output, "Level version: {}", file.graphs.header.version);
    xrf_output::info!(output, "Level graph vertices: {}", file.graphs.header.vertices_count);
    xrf_output::info!(output, "Level graph points: {}", file.graphs.header.points_count);
    xrf_output::info!(output, "Level graph edges: {}", file.graphs.header.edges_count);
  }

  fn print_level(file: &LevelSpawnFile, output: &OutputOptions) {
    let sections: BTreeMap<&str, usize> = file.get_sections();

    xrf_output::info!(output, "Level spawn list information:");
    xrf_output::info!(output, "Objects count: {}", file.objects.len());
    xrf_output::info!(output, "Sections count: {}", sections.len());

    for (section, objects) in sections {
      xrf_output::info!(output, "Section '{}' objects: {}", section, objects);
    }
  }
}
