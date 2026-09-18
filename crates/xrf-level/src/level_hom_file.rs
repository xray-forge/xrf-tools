use std::fs::File;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter, find_required_chunk_by_id};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::level_hom_polygon::HomPolygon;

/// The hierarchical occlusion mesh of a level, `level.hom`.
///
/// Two chunks and nothing else: a version, and a packed run of triangles the renderer builds an AABB tree over.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelHomFile {
  pub version: u32,
  pub polygons: Vec<HomPolygon>,
}

impl LevelHomFile {
  /// The only version shipped, which `CHOM::Load` does not itself check.
  pub const CURRENT_VERSION: u32 = 0;

  pub const VERSION_CHUNK_ID: u32 = 0;

  pub const GEOMETRY_CHUNK_ID: u32 = 1;

  /// Reads an occlusion mesh from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not an occlusion mesh this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level occlusion mesh was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads an occlusion mesh from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not an occlusion mesh this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived mesh arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not an occlusion mesh this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk is absent, or the geometry is not whole triangles.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let version: u32 = find_required_chunk_by_id(&chunks, Self::VERSION_CHUNK_ID)?.read_u32::<T>()?;
    let mut geometry: ChunkReader<D> = find_required_chunk_by_id(&chunks, Self::GEOMETRY_CHUNK_ID)?;

    let count: u64 = geometry.read_bytes_remain();

    if !count.is_multiple_of(HomPolygon::SERIALIZED_SIZE) {
      return Err(XrfError::new_invalid_error(format!(
        "Unexpected level occlusion geometry of {count} bytes, expected whole triangles"
      )));
    }

    let count: u64 = count / HomPolygon::SERIALIZED_SIZE;
    let mut polygons: Vec<HomPolygon> =
      geometry.new_bounded_vec(count, HomPolygon::SERIALIZED_SIZE, "occlusion triangles")?;

    for _ in 0..count {
      polygons.push(geometry.read_xr::<T, _>()?);
    }

    geometry.assert_read("Expect all data to be read from level occlusion geometry chunk")?;

    Ok(Self { version, polygons })
  }

  /// Writes the mesh back in the layout the renderer reads.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut version: ChunkWriter = ChunkWriter::new();

    version.write_u32::<T>(self.version)?;
    version.flush_chunk_into::<T>(&mut writer.buffer, Self::VERSION_CHUNK_ID)?;

    let mut geometry: ChunkWriter = ChunkWriter::new();

    for polygon in &self.polygons {
      geometry.write_xr::<T, _>(polygon)?;
    }

    geometry.flush_chunk_into::<T>(&mut writer.buffer, Self::GEOMETRY_CHUNK_ID)?;

    Ok(())
  }
}
