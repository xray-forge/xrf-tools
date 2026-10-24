use xrf_app_core::TauriResult;
use xrf_chunk::{ChunkReader, InMemoryChunkDataSource, XRayByteOrder};
use xrf_level::{LevelFile, LevelVisualsChunk};
use xrf_vfs::XraySearch;

use crate::level::LevelSource;

/// One compiled level's `level` bundle as it is read: the level and its visuals.
pub struct LevelBundle {
  pub level: LevelFile,
  pub visuals: LevelVisualsChunk,
}

impl LevelBundle {
  /// The file every compiled level has, which is the bundle itself.
  pub const FILE: &'static str = "level";

  /// Reads a level's bundle, whichever way its source names it.
  ///
  /// # Errors
  ///
  /// Returns why the bundle cannot be read, or that it carries no visuals.
  pub fn read(source: &LevelSource, search: &XraySearch) -> TauriResult<Self> {
    let bundle: Vec<u8> = source.read_file(search, Self::FILE)?;
    let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_vec(bundle)
      .and_then(|mut it| it.read_children())
      .map_err(|error| source.describe_read_failure(Self::FILE, &error))?;
    let level: LevelFile = LevelFile::read_from_chunks::<XRayByteOrder, _>(&chunks)
      .map_err(|error| source.describe_read_failure(Self::FILE, &error))?;
    let visuals: LevelVisualsChunk = LevelFile::read_visuals_from_chunks::<XRayByteOrder, _>(&chunks)
      .map_err(|error| source.describe_read_failure(Self::FILE, &error))?
      .ok_or_else(|| {
        format!(
          "Level '{}' carries no visuals chunk, so it draws nothing",
          source.get_label()
        )
      })?;

    Ok(Self { level, visuals })
  }
}
