use std::str::FromStr;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Eq, Display)]
#[serde(rename_all = "camelCase")]
#[display("{x},{y},{z}")]
pub struct Vector3d<T = f32> {
  pub x: T,
  pub y: T,
  pub z: T,
}

impl Vector3d<f32> {
  pub fn new(x: f32, y: f32, z: f32) -> Self {
    Self { x, y, z }
  }
}

impl ChunkReadWrite for Vector3d<f32> {
  /// Read vector coordinates from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      x: reader.read_f32::<T>()?,
      y: reader.read_f32::<T>()?,
      z: reader.read_f32::<T>()?,
    })
  }

  /// Write vector coordinates into the writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_f32::<T>(self.x)?;
    writer.write_f32::<T>(self.y)?;
    writer.write_f32::<T>(self.z)?;

    Ok(())
  }
}

impl From<(f32, f32, f32)> for Vector3d<f32> {
  fn from(value: (f32, f32, f32)) -> Self {
    Vector3d::new(value.0, value.1, value.2)
  }
}

impl FromStr for Vector3d<f32> {
  type Err = XrfError;

  fn from_str(string: &str) -> Result<Self, Self::Err> {
    let parts: Vec<&str> = string.split(',').collect();

    if parts.len() != 3 {
      return Err(XrfError::new_parsing_error(
        "Failed to parse 3d vector from string, expected 3 numbers",
      ));
    }

    Ok(Self {
      x: parts[0]
        .trim()
        .parse::<f32>()
        .or(Err(XrfError::new_parsing_error("Failed to parse vector X value")))?,
      y: parts[1]
        .trim()
        .parse::<f32>()
        .or(Err(XrfError::new_parsing_error("Failed to parse vector Y value")))?,
      z: parts[2]
        .trim()
        .parse::<f32>()
        .or(Err(XrfError::new_parsing_error("Failed to parse vector Z value")))?,
    })
  }
}

#[cfg(any(test, feature = "fixtures"))]
impl Vector3d {
  /// The vector this crate's own tests and its dependants' fixtures build against.
  pub fn new_mock() -> Self {
    Self {
      x: 1.1150,
      y: -0.255,
      z: 1.753,
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write};
  use std::str::FromStr;

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use crate::vector_3d::Vector3d;

  #[test]
  fn test_read_write() -> XrfResult {
    let filename: String = String::from("read_write.chunk");
    let mut writer: ChunkWriter = ChunkWriter::new();

    let original: Vector3d = Vector3d { x: 1.5, y: 2.7, z: 3.2 };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 12);

    let bytes_written: usize = writer.flush_chunk_into::<XRayByteOrder>(
      &mut overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(file!(), &filename))?,
      0,
    )?;

    assert_eq!(bytes_written, 12);

    let file: FileSlice =
      open_generated_test_resource_as_slice(&build_relative_test_sample_file_path(file!(), &filename))?;

    assert_eq!(file.bytes_remaining(), 12 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?
      .read_child_by_index(0)
      .expect("0 index chunk to exist");

    assert_eq!(Vector3d::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_from_to_str() -> XrfResult {
    let original: Vector3d = Vector3d {
      x: 10.5,
      y: 20.7,
      z: 30.2,
    };

    assert_eq!(original.to_string(), "10.5,20.7,30.2");
    assert_eq!(Vector3d::from_str("10.5,20.7,30.2")?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: Vector3d = Vector3d {
      x: 10.5,
      y: 20.7,
      z: 30.2,
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);
    assert_eq!(original, serde_json::from_str::<Vector3d>(&serialized)?);

    Ok(())
  }
}
