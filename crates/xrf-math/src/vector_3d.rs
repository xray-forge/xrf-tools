use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};
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
  pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
  pub const X: Self = Self::new(1.0, 0.0, 0.0);
  pub const Y: Self = Self::new(0.0, 1.0, 0.0);

  pub const fn new(x: f32, y: f32, z: f32) -> Self {
    Self { x, y, z }
  }

  pub fn dot(&self, other: &Self) -> f32 {
    self.x * other.x + self.y * other.y + self.z * other.z
  }

  pub fn cross(&self, other: &Self) -> Self {
    Self::new(
      self.y * other.z - self.z * other.y,
      self.z * other.x - self.x * other.z,
      self.x * other.y - self.y * other.x,
    )
  }

  pub fn length_squared(&self) -> f32 {
    self.dot(self)
  }

  pub fn length(&self) -> f32 {
    self.length_squared().sqrt()
  }

  /// The same direction at unit length, or zero for a vector with none.
  pub fn normalize_or_zero(&self) -> Self {
    let length: f32 = self.length();

    if length > 0.0 { self / length } else { Self::ZERO }
  }

  pub fn to_array(&self) -> [f32; 3] {
    [self.x, self.y, self.z]
  }
}

impl From<[f32; 3]> for Vector3d<f32> {
  fn from([x, y, z]: [f32; 3]) -> Self {
    Self::new(x, y, z)
  }
}

impl Add for &Vector3d<f32> {
  type Output = Vector3d<f32>;

  fn add(self, other: Self) -> Vector3d<f32> {
    Vector3d::new(self.x + other.x, self.y + other.y, self.z + other.z)
  }
}

impl AddAssign<&Vector3d<f32>> for Vector3d<f32> {
  fn add_assign(&mut self, other: &Self) {
    self.x += other.x;
    self.y += other.y;
    self.z += other.z;
  }
}

impl Sub for &Vector3d<f32> {
  type Output = Vector3d<f32>;

  fn sub(self, other: Self) -> Vector3d<f32> {
    Vector3d::new(self.x - other.x, self.y - other.y, self.z - other.z)
  }
}

impl Mul<f32> for &Vector3d<f32> {
  type Output = Vector3d<f32>;

  fn mul(self, scale: f32) -> Vector3d<f32> {
    Vector3d::new(self.x * scale, self.y * scale, self.z * scale)
  }
}

impl Div<f32> for &Vector3d<f32> {
  type Output = Vector3d<f32>;

  fn div(self, divisor: f32) -> Vector3d<f32> {
    Vector3d::new(self.x / divisor, self.y / divisor, self.z / divisor)
  }
}

impl Neg for &Vector3d<f32> {
  type Output = Vector3d<f32>;

  fn neg(self) -> Vector3d<f32> {
    Vector3d::new(-self.x, -self.y, -self.z)
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
  fn test_arithmetic() {
    let a: Vector3d = Vector3d::new(1.0, 2.0, 3.0);
    let b: Vector3d = Vector3d::new(-2.0, 0.5, 4.0);

    assert_eq!(&a + &b, Vector3d::new(-1.0, 2.5, 7.0));
    assert_eq!(&a - &b, Vector3d::new(3.0, 1.5, -1.0));
    assert_eq!(&a * 2.0, Vector3d::new(2.0, 4.0, 6.0));
    assert_eq!(&a / 2.0, Vector3d::new(0.5, 1.0, 1.5));
    assert_eq!(-&a, Vector3d::new(-1.0, -2.0, -3.0));
    assert_eq!(a.dot(&b), 11.0);
    assert_eq!(Vector3d::X.cross(&Vector3d::Y), Vector3d::new(0.0, 0.0, 1.0));
    assert_eq!(Vector3d::new(0.0, 3.0, 4.0).length(), 5.0);
    assert_eq!(
      Vector3d::new(0.0, 3.0, 4.0).normalize_or_zero(),
      Vector3d::new(0.0, 0.6, 0.8)
    );
    assert_eq!(Vector3d::ZERO.normalize_or_zero(), Vector3d::ZERO);

    let mut sum: Vector3d = Vector3d::ZERO;

    sum += &a;
    sum += &b;

    assert_eq!(sum, &a + &b);
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
