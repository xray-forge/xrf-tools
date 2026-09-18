use std::fs::File;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter, find_required_chunk_by_id};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::som::level_som_polygon::SomPolygon;

/// The sound occlusion mesh of a level, `level.som`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSomFile {
  pub version: u32,
  pub polygons: Vec<SomPolygon>,
}

impl LevelSomFile {
  /// The version `CSoundRender_Scene::set_geometry_som` asserts on.
  pub const CURRENT_VERSION: u32 = 0;

  pub const VERSION_CHUNK_ID: u32 = 0;

  pub const GEOMETRY_CHUNK_ID: u32 = 1;

  /// Reads a sound occlusion mesh from a path.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be opened, or is not a sound occlusion mesh this reads.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level sound occlusion mesh was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  /// Reads a sound occlusion mesh from an open file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read, or is not a sound occlusion mesh this reads.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads from bytes already in hand, which is how an archived mesh arrives.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a sound occlusion mesh this reads.
  pub fn read_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_vec(bytes)?)
  }

  /// Reads from a chunk reader over any data source.
  ///
  /// # Errors
  ///
  /// Returns an error when a chunk is absent, the version is one the engine refuses, or the geometry is not whole
  /// triangles.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let version: u32 = find_required_chunk_by_id(&chunks, Self::VERSION_CHUNK_ID)?.read_u32::<T>()?;

    if version != Self::CURRENT_VERSION {
      return Err(XrfError::new_not_implemented_error(format!(
        "Unexpected level sound occlusion version {version} on read, only version {} is implemented",
        Self::CURRENT_VERSION
      )));
    }

    let mut geometry: ChunkReader<D> = find_required_chunk_by_id(&chunks, Self::GEOMETRY_CHUNK_ID)?;
    let count: u64 = geometry.read_bytes_remain();

    if !count.is_multiple_of(SomPolygon::SERIALIZED_SIZE) {
      return Err(XrfError::new_invalid_error(format!(
        "Unexpected level sound occlusion geometry of {count} bytes, expected whole triangles"
      )));
    }

    let count: u64 = count / SomPolygon::SERIALIZED_SIZE;
    let mut polygons: Vec<SomPolygon> =
      geometry.new_bounded_vec(count, SomPolygon::SERIALIZED_SIZE, "sound occlusion triangles")?;

    for _ in 0..count {
      polygons.push(geometry.read_xr::<T, _>()?);
    }

    geometry.assert_read("Expect all data to be read from level sound occlusion geometry chunk")?;

    Ok(Self { version, polygons })
  }

  /// Writes the mesh back in the layout the sound renderer reads.
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

impl LevelSomFile {
  /// Faces the loader builds, which is more than the triangle count wherever an occluder is two-sided.
  pub fn get_faces_count(&self) -> usize {
    self.polygons.iter().map(SomPolygon::get_faces_count).sum()
  }

  /// Triangles that occlude from both sides.
  pub fn get_two_sided_count(&self) -> usize {
    self.polygons.iter().filter(|polygon| polygon.is_two_sided != 0).count()
  }
}
