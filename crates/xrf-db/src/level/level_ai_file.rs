use std::fs::File;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::data::generic::vector_3d::Vector3d;

/// `hdrNODES` in c++ codebase, stored raw at the very start of the `level.ai` file.
///
/// Unlike most xray formats `level.ai` is not chunked - `CLevelGraph` opens the file and casts its
/// pointer straight onto this structure, so the header occupies the first 56 bytes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelAiHeader {
  pub version: u32,
  pub count: u32,
  pub size: f32,
  pub size_y: f32,
  pub aabb_min: Vector3d<f32>,
  pub aabb_max: Vector3d<f32>,
  pub guid: Uuid,
}

impl LevelAiHeader {
  /// Byte size of the header as laid out by the engine.
  pub const SIZE: u64 = 56;
}

impl ChunkReadWrite for LevelAiHeader {
  /// Read level AI-map header from the chunk reader.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      version: reader.read_u32::<T>()?,
      count: reader.read_u32::<T>()?,
      size: reader.read_f32::<T>()?,
      size_y: reader.read_f32::<T>()?,
      aabb_min: reader.read_xr::<T, _>()?,
      aabb_max: reader.read_xr::<T, _>()?,
      guid: Uuid::from_u128(reader.read_u128::<T>()?),
    })
  }

  /// Write level AI-map header into the chunk writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.version)?;
    writer.write_u32::<T>(self.count)?;
    writer.write_f32::<T>(self.size)?;
    writer.write_f32::<T>(self.size_y)?;
    writer.write_xr::<T, _>(&self.aabb_min)?;
    writer.write_xr::<T, _>(&self.aabb_max)?;
    writer.write_u128::<T>(self.guid.as_u128())?;

    Ok(())
  }
}

/// Descriptor of the `level.ai` file used by xray game engine.
///
/// Only the header is read. Node payload is not parsed - it is the single largest file in a level
/// bundle and nothing in it can be validated without reimplementing the node compressor.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelAiFile {
  pub header: LevelAiHeader,
  // todo: Content.
}

impl LevelAiFile {
  /// Read level AI-map file from provided path.
  pub fn read_from_path<T: ByteOrder, P: AsRef<Path>>(path: &P) -> XrfResult<Self> {
    Self::read_from_file::<T>(File::open(path).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Level AI-map file was not read: {}, error: {}",
        format_path(path.as_ref()),
        error
      ))
    })?)
  }

  /// Read level AI-map file from file.
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
}

#[cfg(test)]
mod tests {
  use std::io::Write;

  use uuid::uuid;
  use xrf_chunk::{ChunkReadWrite, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_file,
    overwrite_generated_test_resource_as_file,
  };

  use crate::data::generic::vector_3d::Vector3d;
  use crate::level::level_ai_file::{LevelAiFile, LevelAiHeader};

  fn sample() -> LevelAiHeader {
    LevelAiHeader {
      version: 10,
      count: 1_356_204,
      size: 0.7,
      size_y: 0.2,
      aabb_min: Vector3d::new(-600.0, -20.5, -615.0),
      aabb_max: Vector3d::new(600.0, 80.25, 585.0),
      guid: uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7"),
    }
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.ai");
    let mut writer: ChunkWriter = ChunkWriter::new();
    let original: LevelAiHeader = sample();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written() as u64, LevelAiHeader::SIZE);

    writer.flush_raw_into(&mut overwrite_generated_test_resource_as_file(
      &build_relative_test_sample_file_path(file!(), &filename),
    )?)?;

    let read: LevelAiFile = LevelAiFile::read_from_file::<XRayByteOrder>(open_generated_test_resource_as_file(
      &build_relative_test_sample_file_path(file!(), &filename),
    )?)?;

    assert_eq!(read.header, original);

    Ok(())
  }

  #[test]
  fn truncated_header_is_an_error_and_not_a_panic() -> XrfResult {
    let filename: String = String::from("truncated.ai");
    let mut writer: ChunkWriter = ChunkWriter::new();

    sample().write::<XRayByteOrder>(&mut writer)?;

    let mut bytes: Vec<u8> = writer.flush_raw_into_buffer()?;

    bytes.truncate(LevelAiHeader::SIZE as usize - 1);

    overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?
      .write_all(&bytes)?;

    assert!(
      LevelAiFile::read_from_file::<XRayByteOrder>(open_generated_test_resource_as_file(
        &build_relative_test_sample_file_path(file!(), &filename)
      )?)
      .is_err(),
      "Expected truncated AI-map header to fail reading"
    );

    Ok(())
  }
}
