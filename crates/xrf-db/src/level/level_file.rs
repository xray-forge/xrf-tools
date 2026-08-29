use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, find_optional_chunk_by_id, find_required_chunk_by_id};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::level::level_header_chunk::LevelHeaderChunk;
use crate::level::level_shaders_chunk::LevelShadersChunk;

/// Descriptor of the compiled `level` file used by xray game engine.
///
/// Only the chunks needed to validate a built level bundle are read. Geometry, portals, sectors,
/// glows and lights are left untouched - the engine streams them lazily and they are orders of
/// magnitude larger than the rest of the file.
///
/// Root level chunks by ID:
/// 1 - header
/// 2 - shaders
/// 3 - visuals
/// 4 - portals
/// 6 - dynamic light
/// 7 - glows
/// 8 - sectors
/// 9 - vertex buffer
/// 10 - index buffer
/// 11 - slide window items
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelFile {
  pub header: LevelHeaderChunk,
  /// Absent shaders chunk is a fatal defect rather than a read failure, so it is reported as data
  /// instead of an error. The renderer asserts on it with `Level doesn't builded correctly.` -
  /// quoted verbatim from `r2_loader.cpp` so it matches what the engine prints to the log.
  pub shaders: Option<LevelShadersChunk>,
}

impl LevelFile {
  /// Read level file from provided path.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level file was not read: {}, error: {}",
        format_path(path.as_ref()),
        error
      ))
    })?)
  }

  /// Read level file from file.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// The route an archived entry takes: a volume holds no file to slice, only bytes.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    Self::read_from_chunks::<T, _>(&chunks)
  }

  /// Read level file from chunks.
  pub fn read_from_chunks<T: ByteOrder, D: ChunkDataSource>(chunks: &[ChunkReader<D>]) -> XrfResult<Self> {
    Ok(Self {
      header: find_required_chunk_by_id(chunks, LevelHeaderChunk::CHUNK_ID)?.read_xr::<T, _>()?,
      shaders: match find_optional_chunk_by_id(chunks, LevelShadersChunk::CHUNK_ID) {
        Some(mut it) => Some(it.read_xr::<T, _>()?),
        None => None,
      },
    })
  }
}
