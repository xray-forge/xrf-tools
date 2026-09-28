use std::fs::File;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;
use xrf_utils::format_path;

use crate::cform::level_cform_cover::LevelCformCover;
use crate::cform::level_cform_geometry::LevelCformGeometry;

/// `hdrCFORM` in c++ codebase, stored raw at the very start of the `level.cform` file.
///
/// Like `level.ai` the collision form file is not chunked - `CObjectSpace` reads the header
/// directly, so it occupies the first 36 bytes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelCformHeader {
  pub version: u32,
  pub vertex_count: u32,
  pub face_count: u32,
  pub aabb_min: Vector3d<f32>,
  pub aabb_max: Vector3d<f32>,
}

impl LevelCformHeader {
  /// Byte size of the header as laid out by the engine.
  pub const SIZE: u64 = 36;
}

impl ChunkReadWrite for LevelCformHeader {
  /// Read level collision form header from the chunk reader.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      version: reader.read_u32::<T>()?,
      vertex_count: reader.read_u32::<T>()?,
      face_count: reader.read_u32::<T>()?,
      aabb_min: reader.read_xr::<T, _>()?,
      aabb_max: reader.read_xr::<T, _>()?,
    })
  }

  /// Write level collision form header into the chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.version)?;
    writer.write_u32::<T>(self.vertex_count)?;
    writer.write_u32::<T>(self.face_count)?;
    writer.write_xr::<T, _>(&self.aabb_min)?;
    writer.write_xr::<T, _>(&self.aabb_max)?;

    Ok(())
  }
}

/// Descriptor of the `level.cform` file used by xray game engine.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelCformFile {
  pub header: LevelCformHeader,
}

impl LevelCformFile {
  /// Read level collision form file from provided path.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level collision form file was not read: {}, error: {}",
        format_path(path.as_ref()),
        error
      ))
    })?)
  }

  /// Read level collision form file from file.
  pub fn read_from_file<T: ByteOrder>(file: File) -> XrfResult<Self> {
    Self::read_from_chunk::<T, _>(&mut ChunkReader::from_file(file)?)
  }

  /// Reads the header from a chunk reader over any data source.
  ///
  /// The route an archived level file takes: a volume holds no file to slice, only bytes.
  pub fn read_from_chunk<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      header: reader.read_xr::<T, _>()?,
    })
  }

  /// Reads the header and the vertices and faces it counts, which only a consumer walking the collision form wants.
  pub fn read_with_geometry_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<(Self, LevelCformGeometry)> {
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_vec(bytes)?;
    let file: Self = Self::read_from_chunk::<T, _>(&mut reader)?;
    let geometry: LevelCformGeometry = LevelCformGeometry::read_from_chunk::<T, _>(&mut reader, &file.header)?;

    Ok((file, geometry))
  }

  /// Whether no face lies straight above each point, the faces tested as they are read rather than kept.
  pub fn read_open_above_from_bytes<T: ByteOrder>(bytes: Vec<u8>, points: &[Vector3d<f32>]) -> XrfResult<Vec<bool>> {
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_vec(bytes)?;
    let file: Self = Self::read_from_chunk::<T, _>(&mut reader)?;

    LevelCformCover::read_open_above::<T, _>(&mut reader, &file.header, points)
  }

  /// Reads the vertices and faces alone, the header read only for the counts it holds.
  pub fn read_geometry_from_bytes<T: ByteOrder>(bytes: Vec<u8>) -> XrfResult<LevelCformGeometry> {
    Self::read_with_geometry_from_bytes::<T>(bytes).map(|(_, geometry)| geometry)
  }
}
