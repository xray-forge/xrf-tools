use xrf_chunk::{ChunkReader, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelFile, LevelVisualsChunk};
use xrf_vfs::XraySearch;

use crate::level::LevelSource;

/// One compiled level's `level` bundle as it is read: the level and its visuals.
pub struct LevelBundle {
  pub level: LevelFile,
  pub visuals: LevelVisualsChunk,
}

impl LevelBundle {
  /// Reads a level's bundle, whichever way its source names it.
  ///
  /// # Errors
  ///
  /// Returns why the bundle cannot be read, or that it carries no visuals.
  pub(crate) fn read(source: &LevelSource, search: &XraySearch) -> XrfResult<Self> {
    let bundle: Vec<u8> = source.read_file(search, LevelFile::FILE_NAME)?;
    let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_vec(bundle)
      .and_then(|mut it| it.read_children())
      .map_err(|error| source.create_read_error(LevelFile::FILE_NAME, &error))?;
    let level: LevelFile =
      LevelFile::read_from_chunks(&chunks).map_err(|error| source.create_read_error(LevelFile::FILE_NAME, &error))?;
    let visuals: LevelVisualsChunk = LevelFile::read_visuals_from_chunks(&chunks)
      .map_err(|error| source.create_read_error(LevelFile::FILE_NAME, &error))?
      .ok_or_else(|| {
        XrfError::new_not_found_error(format!(
          "Level '{}' carries no visuals chunk, so it draws nothing",
          source.get_label()
        ))
      })?;

    Ok(Self { level, visuals })
  }
}
