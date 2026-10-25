use std::fs::File;
use std::path::Path;

use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::level::dynamic_lights::level_dynamic_lights_chunk::LevelDynamicLightsChunk;
use crate::level::level_header_chunk::LevelHeaderChunk;
use crate::level::portals::level_portal::LevelPortal;
use crate::level::portals::level_portals_chunk::LevelPortalsChunk;
use crate::level::sectors::level_sector::LevelSector;
use crate::level::sectors::level_sectors_chunk::LevelSectorsChunk;
use crate::level::shaders::level_shaders_chunk::LevelShadersChunk;
use crate::level::visuals::level_visuals_chunk::LevelVisualsChunk;

/// Descriptor of the compiled `level` file used by xray game engine.
///
/// Root level chunks by ID:
/// 1 - header
/// 2 - shaders
/// 3 - visuals
/// 4 - portals
/// 6 - dynamic light
/// 7 - glows
/// 8 - sectors
///
/// The buffers the visuals draw from (chunks 9 to 11) live in `level.geom`, read by [`crate::LevelGeomFile`].
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelFile {
  pub header: LevelHeaderChunk,
  /// Absent shaders chunk is a fatal defect rather than a read failure, so it is reported as data
  /// instead of an error. The renderer asserts on it with `Level doesn't builded correctly.` -
  /// quoted verbatim from `r2_loader.cpp` so it matches what the engine prints to the log.
  pub shaders: Option<LevelShadersChunk>,
  /// The sectors the renderer culls by, absent for a level built without them.
  pub sectors: Option<LevelSectorsChunk>,
  /// The portals sectors see each other through.
  pub portals: Option<LevelPortalsChunk>,
  /// The static lights the renderer loads, one of which may be the level's sun.
  pub lights: Option<LevelDynamicLightsChunk>,
}

impl LevelFile {
  /// The file a compiled level's bundle is shipped as, in its directory.
  pub const FILE_NAME: &'static str = "level";

  /// The sectors the level names, none for a level built without them.
  pub fn get_sectors(&self) -> &[LevelSector] {
    self.sectors.as_ref().map_or(&[], |chunk| &chunk.sectors)
  }

  /// The portals the level names, none for a level built without them.
  pub fn get_portals(&self) -> &[LevelPortal] {
    self.portals.as_ref().map_or(&[], |chunk| &chunk.portals)
  }

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

  /// Reads from bytes already in hand, which is how an archived bundle arrives: a volume holds no file to open.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// The route an archived entry takes: a volume holds no file to slice, only bytes.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    Self::read_from_chunks::<T, _>(&chunks)
  }

  /// Read every visual of a level from its path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened or a visual cannot be read.
  pub fn read_visuals_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Option<LevelVisualsChunk>> {
    Self::read_visuals_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level file was not read: {}, error: {}",
        format_path(path.as_ref()),
        error
      ))
    })?)
  }

  /// Read every visual of a level from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read or a visual cannot be read.
  pub fn read_visuals_from_file<T: ByteOrder>(file: File) -> XrfResult<Option<LevelVisualsChunk>> {
    Self::read_visuals_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Read every visual of a level from bytes already in hand, which is how an archived level arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a level, or a visual cannot be read.
  pub fn read_visuals_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Option<LevelVisualsChunk>> {
    Self::read_visuals_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Read every visual of a level from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when the source cannot be read or a visual cannot be read.
  pub fn read_visuals_from_chunk<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
  ) -> XrfResult<Option<LevelVisualsChunk>> {
    Self::read_visuals_from_chunks::<T, _>(&reader.read_children()?)
  }

  /// Read every visual of a level from chunks already read from it.
  ///
  /// # Errors
  ///
  /// Returns an error when a visual cannot be read.
  pub fn read_visuals_from_chunks<T: ByteOrder, D: ChunkDataSource>(
    chunks: &[ChunkReader<D>],
  ) -> XrfResult<Option<LevelVisualsChunk>> {
    match ChunkReader::find_optional_by_id(chunks, LevelVisualsChunk::CHUNK_ID) {
      Some(mut chunk) => Ok(Some(LevelVisualsChunk::read_from_chunk::<T, _>(&mut chunk)?)),
      None => Ok(None),
    }
  }

  /// Read level file from chunks.
  pub fn read_from_chunks<T: ByteOrder, D: ChunkDataSource>(chunks: &[ChunkReader<D>]) -> XrfResult<Self> {
    Ok(Self {
      header: ChunkReader::find_required_by_id(chunks, LevelHeaderChunk::CHUNK_ID)?.read_xr::<T, _>()?,
      lights: match ChunkReader::find_optional_by_id(chunks, LevelDynamicLightsChunk::CHUNK_ID) {
        Some(mut it) => Some(LevelDynamicLightsChunk::read_from_chunk::<T, _>(&mut it)?),
        None => None,
      },
      portals: match ChunkReader::find_optional_by_id(chunks, LevelPortalsChunk::CHUNK_ID) {
        Some(mut it) => Some(LevelPortalsChunk::read_from_chunk::<T, _>(&mut it)?),
        None => None,
      },
      sectors: match ChunkReader::find_optional_by_id(chunks, LevelSectorsChunk::CHUNK_ID) {
        Some(mut it) => Some(LevelSectorsChunk::read_from_chunk::<T, _>(&mut it)?),
        None => None,
      },
      shaders: match ChunkReader::find_optional_by_id(chunks, LevelShadersChunk::CHUNK_ID) {
        Some(mut it) => Some(it.read_xr::<T, _>()?),
        None => None,
      },
    })
  }
}
