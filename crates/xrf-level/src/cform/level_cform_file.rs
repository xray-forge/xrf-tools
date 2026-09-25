use std::fs::File;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;
use xrf_utils::format_path;

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
///
/// Only the header is read. Vertex and triangle payload is not parsed - it is validated by the
/// engine's collision database loader and cannot be checked cheaply.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelCformFile {
  pub header: LevelCformHeader,
  // todo: Content.
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
}

#[cfg(test)]
mod tests {
  use std::io::Write;

  use xrf_chunk::{ChunkReadWrite, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_math::Vector3d;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_file,
    overwrite_generated_test_resource_as_file,
  };

  use crate::cform::level_cform_face::LevelCformFace;
  use crate::cform::level_cform_file::{LevelCformFile, LevelCformHeader};

  fn sample() -> LevelCformHeader {
    LevelCformHeader {
      version: 4,
      vertex_count: 420_690,
      face_count: 810_223,
      aabb_min: Vector3d::new(-600.0, -20.5, -615.0),
      aabb_max: Vector3d::new(600.0, 80.25, 585.0),
    }
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.cform");
    let mut writer: ChunkWriter = ChunkWriter::new();
    let original: LevelCformHeader = sample();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written() as u64, LevelCformHeader::SIZE);

    writer.flush_raw_into(&mut overwrite_generated_test_resource_as_file(
      &build_relative_test_sample_file_path(file!(), &filename),
    )?)?;

    let read: LevelCformFile = LevelCformFile::read_from_file::<XRayByteOrder>(open_generated_test_resource_as_file(
      &build_relative_test_sample_file_path(file!(), &filename),
    )?)?;

    assert_eq!(read.header, original);

    Ok(())
  }

  #[test]
  fn reads_the_vertices_and_faces_the_header_counts() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let header: LevelCformHeader = LevelCformHeader {
      version: 4,
      vertex_count: 3,
      face_count: 1,
      aabb_min: Vector3d::new(0.0, 0.0, 0.0),
      aabb_max: Vector3d::new(1.0, 0.0, 1.0),
    };

    header.write::<XRayByteOrder>(&mut writer)?;

    let mut bytes: Vec<u8> = writer.flush_raw_into_buffer()?;

    for value in [0.0_f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0] {
      bytes.extend_from_slice(&value.to_le_bytes());
    }

    for index in [0_u32, 2, 1] {
      bytes.extend_from_slice(&index.to_le_bytes());
    }

    // Material 5, shadows suppressed, sector 7.
    bytes.extend_from_slice(&(5_u32 | (1 << 14) | (7 << 16)).to_le_bytes());

    let (file, geometry) = LevelCformFile::read_with_geometry_from_bytes::<XRayByteOrder>(bytes)?;
    let face: LevelCformFace = geometry.faces[0];

    assert_eq!(file.header, header);
    assert_eq!(geometry.vertices[1], Vector3d::new(1.0, 0.0, 0.0));
    assert_eq!(face.vertices, [0, 2, 1]);
    assert_eq!(face.material, 5);
    assert!(face.is_shadow_suppressed);
    assert!(!face.is_wallmark_suppressed);
    assert_eq!(face.sector, 7);
    assert_eq!(geometry.get_triangle(&face)[1], Vector3d::new(0.0, 0.0, 1.0));

    Ok(())
  }

  #[test]
  fn a_face_naming_a_vertex_past_the_count_is_an_error() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    LevelCformHeader {
      version: 4,
      vertex_count: 1,
      face_count: 1,
      aabb_min: Vector3d::new(0.0, 0.0, 0.0),
      aabb_max: Vector3d::new(0.0, 0.0, 0.0),
    }
    .write::<XRayByteOrder>(&mut writer)?;

    let mut bytes: Vec<u8> = writer.flush_raw_into_buffer()?;

    bytes.extend_from_slice(&[0; 12]);

    for index in [0_u32, 0, 3, 0] {
      bytes.extend_from_slice(&index.to_le_bytes());
    }

    assert!(LevelCformFile::read_with_geometry_from_bytes::<XRayByteOrder>(bytes).is_err());

    Ok(())
  }

  #[test]
  fn truncated_header_is_an_error_and_not_a_panic() -> XrfResult {
    let filename: String = String::from("truncated.cform");
    let mut writer: ChunkWriter = ChunkWriter::new();

    sample().write::<XRayByteOrder>(&mut writer)?;

    let mut bytes: Vec<u8> = writer.flush_raw_into_buffer()?;

    bytes.truncate(LevelCformHeader::SIZE as usize - 1);

    overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?
      .write_all(&bytes)?;

    assert!(
      LevelCformFile::read_from_file::<XRayByteOrder>(open_generated_test_resource_as_file(
        &build_relative_test_sample_file_path(file!(), &filename)
      )?)
      .is_err(),
      "Expected truncated collision form header to fail reading"
    );

    Ok(())
  }
}
