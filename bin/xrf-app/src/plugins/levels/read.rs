use std::fs;
use std::path::Path;

use xrf_chunk::InMemoryChunkDataSource;
use xrf_level::{LevelFile, LevelGeomSource, LevelVisualsChunk};
use xrf_spawn::XRayByteOrder;
use xrf_vfs::{XrayProbe, XrayResolution};

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

  let level: LevelFile =
    LevelFile::read_from_bytes::<XRayByteOrder>(bundle.clone()).map_err(|error| failure(source, LEVEL_FILE, &error))?;

  let visuals: LevelVisualsChunk = LevelFile::read_visuals_from_bytes::<XRayByteOrder>(bundle)
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
fn read_file(source: &LevelSource, probe: &XrayProbe, file: &str) -> TauriResult<Vec<u8>> {
  match source {
    LevelSource::Directory { path } => {
      let path: std::path::PathBuf = Path::new(path).join(file);

      fs::read(&path).map_err(|error| format!("Failed to read level file '{}': {error}", path.display()))
    }
    LevelSource::Asset { logical_path } => read_asset(probe, &format!("{logical_path}\\{file}")),
  }
}

/// Reads one of the level's files out of the mounted roots, loose or archived alike.
fn read_asset(probe: &XrayProbe, logical_path: &str) -> TauriResult<Vec<u8>> {
  let resolution: XrayResolution = probe
    .find(logical_path)
    .map_err(|error| format!("Rejected level file '{logical_path}': {error}"))?;

  let Some(asset) = resolution.get_asset() else {
    return Err(format!(
      "Failed to read level file '{logical_path}': it resolves to nothing"
    ));
  };

  probe
    .read_asset_bytes(asset)
    .map_err(|error| format!("Failed to read level file '{logical_path}': {error}"))
}

fn failure(source: &LevelSource, file: &str, error: &impl std::fmt::Display) -> String {
  format!("Failed to read '{file}' of level '{}': {error}", source.get_label())
}
