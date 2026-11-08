use std::str::FromStr;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};

/// Two named components, the engine's `Fvector2`: a texture coordinate, a sprite's size on its sheet.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, Display)]
#[serde(rename_all = "camelCase")]
#[display("{x},{y}")]
pub struct Vector2d {
  pub x: f32,
  pub y: f32,
}

impl Vector2d {
  pub const ZERO: Self = Self::new(0.0, 0.0);

  /// Two `f32`s.
  pub const SERIALIZED_SIZE: u64 = 8;

  pub const fn new(x: f32, y: f32) -> Self {
    Self { x, y }
  }

  pub const fn to_array(&self) -> [f32; 2] {
    [self.x, self.y]
  }
}

impl From<[f32; 2]> for Vector2d {
  fn from([x, y]: [f32; 2]) -> Self {
    Self::new(x, y)
  }
}

impl ChunkReadWrite for Vector2d {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      x: reader.read_f32::<T>()?,
      y: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_f32::<T>(self.x)?;
    writer.write_f32::<T>(self.y)?;

    Ok(())
  }
}

impl FromStr for Vector2d {
  type Err = XrfError;

  /// Reads `x,y`, whitespace around each number allowed, as [`Display`] writes it.
  fn from_str(string: &str) -> Result<Self, Self::Err> {
    let parse = |part: &str, axis: &str| {
      part
        .trim()
        .parse::<f32>()
        .map_err(|_| XrfError::new_parsing_error(format!("Failed to parse vector {axis} value '{}'", part.trim())))
    };

    match string.split(',').collect::<Vec<&str>>().as_slice() {
      [x, y] => Ok(Self::new(parse(x, "X")?, parse(y, "Y")?)),
      _ => Err(XrfError::new_parsing_error(
        "Failed to parse 2d vector from string, expected 2 numbers",
      )),
    }
  }
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;

  use crate::vector_2d::Vector2d;

  #[test]
  fn reads_back_what_it_writes() -> XrfResult {
    let original: Vector2d = Vector2d::new(0.25, -1.5);
    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written() as u64, Vector2d::SERIALIZED_SIZE);
    assert_eq!(
      Vector2d::read::<XRayByteOrder, _>(&mut ChunkReader::from_bytes(&writer.buffer)?)?,
      original
    );

    Ok(())
  }

  #[test]
  fn parses_what_it_displays_and_refuses_another_count() -> XrfResult {
    let original: Vector2d = Vector2d::new(10.5, 20.75);

    assert_eq!(original.to_string(), "10.5,20.75");
    assert_eq!(Vector2d::from_str(" 10.5 , 20.75 ")?, original);
    assert!(Vector2d::from_str("1,2,3").is_err());
    assert!(Vector2d::from_str("1,y").is_err());

    Ok(())
  }
}
