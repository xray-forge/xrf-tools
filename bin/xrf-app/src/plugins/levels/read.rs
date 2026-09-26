use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use xrf_chunk::{ChunkReader, InMemoryChunkDataSource};
use xrf_level::{LevelFile, LevelGeomSource, LevelVisualsChunk};
use xrf_spawn::XRayByteOrder;
use xrf_vfs::{XrayLogicalPath, XrayProbe, XrayResolution};

use crate::core::types::TauriResult;
use crate::plugins::levels::state::{GEOMETRY_FILE, LEVEL_FILE, LevelSource};

/// What one compiled level is read as: the bundle, its visuals, and geometry left where it lies.
pub struct ReadLevel {
  pub level: LevelFile,
  pub visuals: LevelVisualsChunk,
  pub geometry: LevelGeomSource<InMemoryChunkDataSource>,
}

/// Reads a level, whichever way its source names it.
pub fn read_source(source: &LevelSource, probe: &XrayProbe) -> TauriResult<ReadLevel> {
  let bundle: Vec<u8> = read_file(source, probe, LEVEL_FILE)?;
  let geometry: Vec<u8> = read_file(source, probe, GEOMETRY_FILE)?;

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
pub fn read_file(source: &LevelSource, probe: &XrayProbe, file: &str) -> TauriResult<Vec<u8>> {
  read_optional_file(source, probe, file)?.ok_or_else(|| format!("Level '{}' has no '{file}'", source.get_label()))
}

/// Reads one of the level's files, or `None` where the level has none: only a file that is not there, not one that
/// cannot be read.
pub fn read_optional_file(source: &LevelSource, probe: &XrayProbe, file: &str) -> TauriResult<Option<Vec<u8>>> {
  match source {
    LevelSource::Directory { path } => {
      let path: PathBuf = Path::new(path).join(file);

      match fs::read(&path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("Failed to read level file '{}': {error}", path.display())),
      }
    }
    LevelSource::Asset { logical_path } => {
      let logical_path: XrayLogicalPath = XrayLogicalPath::new(logical_path)
        .and_then(|directory| directory.join(file))
        .map_err(|error| format!("Rejected level file '{file}' of '{logical_path}': {error}"))?;
      let resolution: XrayResolution = probe
        .find(logical_path.as_str())
        .map_err(|error| format!("Rejected level file '{logical_path}': {error}"))?;

      resolution
        .get_asset()
        .map(|asset| probe.read_asset_bytes(asset))
        .transpose()
        .map_err(|error| format!("Failed to read level file '{logical_path}': {error}"))
    }
  }
}

fn failure(source: &LevelSource, file: &str, error: &impl std::fmt::Display) -> String {
  format!("Failed to read '{file}' of level '{}': {error}", source.get_label())
}
