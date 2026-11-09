use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReadWrite, ChunkFieldReader, ChunkFieldWriter};
use xrf_error::XrfResult;

/// A colour of four floats, the engine's `Fcolor`, stored red, green, blue, alpha. Not a packed `D3DCOLOR`, which is
/// four bytes in another order.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Color {
  pub r: f32,
  pub g: f32,
  pub b: f32,
  pub a: f32,
}

impl Color {
  /// Four `f32`s.
  pub const SERIALIZED_SIZE: u64 = 16;

  pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
    Self { r, g, b, a }
  }
}

/// Four floats, red first.
impl ChunkFieldReadWrite for Color {
  #[inline]
  fn read_fields<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    Ok(Self {
      r: reader.read_f32::<T>()?,
      g: reader.read_f32::<T>()?,
      b: reader.read_f32::<T>()?,
      a: reader.read_f32::<T>()?,
    })
  }

  #[inline]
  fn write_fields<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    writer.write_f32::<T>(self.r)?;
    writer.write_f32::<T>(self.g)?;
    writer.write_f32::<T>(self.b)?;
    writer.write_f32::<T>(self.a)
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;

  use crate::color::Color;

  #[test]
  fn reads_back_what_it_writes_red_first() -> XrfResult {
    let original: Color = Color::new(0.25, 0.5, 0.75, 1.0);
    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written() as u64, Color::SERIALIZED_SIZE);
    assert_eq!(&writer.buffer[..4], &0.25_f32.to_le_bytes());
    assert_eq!(
      Color::read::<XRayByteOrder, _>(&mut ChunkReader::from_bytes(&writer.buffer)?)?,
      original
    );

    Ok(())
  }
}
