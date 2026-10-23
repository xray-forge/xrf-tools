use xrf_chunk::{ChunkReader, InMemoryChunkDataSource, XRayByteOrder};
use xrf_level::{LevelFile, LevelGeomSource, LevelVisualsChunk};
use xrf_vfs::{XrayAsset, XrayLogicalPath, XrayRootProbe, XraySearch};

use crate::core::types::TauriResult;
use crate::plugins::levels::state::{GEOMETRY_FILE, LEVEL_FILE, LevelSource};

/// What one compiled level is read as: the bundle, its visuals, and geometry left where it lies.
pub struct ReadLevel {
  pub level: LevelFile,
  pub visuals: LevelVisualsChunk,
  pub geometry: LevelGeomSource<InMemoryChunkDataSource>,
}

/// Reads a level, whichever way its source names it.
pub fn read_source(source: &LevelSource, search: &XraySearch) -> TauriResult<ReadLevel> {
  let bundle: Vec<u8> = read_file(source, search, LEVEL_FILE)?;
  let geometry: Vec<u8> = read_file(source, search, GEOMETRY_FILE)?;

  let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_vec(bundle)
    .and_then(|mut it| it.read_children())
    .map_err(|error| failure(source, LEVEL_FILE, &error))?;
  let level: LevelFile =
    LevelFile::read_from_chunks::<XRayByteOrder, _>(&chunks).map_err(|error| failure(source, LEVEL_FILE, &error))?;

  let visuals: LevelVisualsChunk = LevelFile::read_visuals_from_chunks::<XRayByteOrder, _>(&chunks)
    .map_err(|error| failure(source, LEVEL_FILE, &error))?
    .ok_or_else(|| {
      format!(
        "Level '{}' carries no visuals chunk, so it draws nothing",
        source.get_label()
      )
    })?;

  Ok(ReadLevel {
    geometry: LevelGeomSource::open_from_bytes::<XRayByteOrder>(geometry)
      .map_err(|error| failure(source, GEOMETRY_FILE, &error))?,
    level,
    visuals,
  })
}

/// Reads one of the level's files, from disk or out of the mounted roots.
pub fn read_file(source: &LevelSource, search: &XraySearch, file: &str) -> TauriResult<Vec<u8>> {
  read_optional_file(source, search, file)?.ok_or_else(|| format!("Level '{}' has no '{file}'", source.get_label()))
}

/// Reads one of the level's files through `search`, or `None` where the level has none: only a file that is not there,
/// not one that cannot be read.
pub fn read_optional_file(source: &LevelSource, search: &XraySearch, file: &str) -> TauriResult<Option<Vec<u8>>> {
  let logical_path: XrayLogicalPath = to_level_logical_path(source, file)?;
  let found: Option<XrayAsset> = search
    .find(logical_path.as_str())
    .map_err(|error| format!("Rejected level file '{logical_path}': {error}"))?;

  found
    .map(|asset| search.read_asset(&asset))
    .transpose()
    .map_err(|error| format!("Failed to read level file '{logical_path}': {error}"))
}

/// A level file's engine identity, which the search reads it by: a level opened as a directory is read by the logical
/// path its implied root gives it, which the open's roots, centred on it, search first, so its reads are recorded as an
/// archived level's are.
///
/// # Errors
///
/// Returns why the file was refused: a name that would leave the level's directory (a root, a drive, `.` or `..`, as
/// a spawn's motion names can be written), or a directory under no root a search would mount.
pub fn to_level_logical_path(source: &LevelSource, file: &str) -> TauriResult<XrayLogicalPath> {
  let directory: XrayLogicalPath = match source {
    LevelSource::Asset { logical_path } => {
      XrayLogicalPath::new(logical_path).map_err(|error| format!("Rejected level '{logical_path}': {error}"))?
    }
    LevelSource::Directory { path } => XrayRootProbe::implied_logical_path(path).ok_or_else(|| {
      format!("Level directory '{path}' lies under no game data root, so no search of its roots reaches it")
    })?,
  };

  if file.is_empty() {
    return Err(format!("Rejected a level file of '{directory}' named by nothing"));
  }

  directory.join(file).map_err(|error| {
    format!("Rejected level file '{file}' of '{directory}': it names no file inside the level: {error}")
  })
}

fn failure(source: &LevelSource, file: &str, error: &impl std::fmt::Display) -> String {
  format!("Failed to read '{file}' of level '{}': {error}", source.get_label())
}
