use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::{AnimationInterpolation, AnimationShape};

/// One key of an animation envelope, `st_Key`: a value at a time, and the curve leading to it. Zero by default, as
/// `st_Key::st_Key` leaves it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationKey {
  pub value: f32,
  /// Seconds from the start of the animation.
  pub time: f32,
  pub shape: AnimationShape,
  /// Carried by every key, as `st_Key` carries it, though the narrow layout stores it only for a key that is not
  /// stepped: a stepped key written that way reads back with zeros here.
  pub interpolation: AnimationInterpolation,
}

impl AnimationKey {
  /// Bytes a stepped key occupies: the value, the time and the shape.
  pub const MIN_SERIALIZED_SIZE: u64 = 4 + 4 + 1;

  /// Bytes a wide key occupies, including the seven unquantized interpolation parameters.
  pub(crate) const WIDE_SERIALIZED_SIZE: u64 = 4 + 4 + 4 + 7 * 4;

  /// Reads a key the narrow way, `st_Key::Load_2`.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload ends inside the key.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let value: f32 = reader.read_f32::<T>()?;
    let time: f32 = reader.read_f32::<T>()?;
    let shape: AnimationShape = AnimationShape::from(reader.read_u8()?);

    Ok(Self {
      value,
      time,
      shape,
      interpolation: if shape.is_stepped() {
        AnimationInterpolation::default()
      } else {
        AnimationInterpolation::read::<T, D>(reader)?
      },
    })
  }

  /// Reads a key the wide way, `st_Key::Load_1`, which stores the curve parameters of a stepped key too.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload ends inside the key.
  pub fn read_wide<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let value: f32 = reader.read_f32::<T>()?;
    let time: f32 = reader.read_f32::<T>()?;
    // Narrowed the way the engine does, which keeps only the low byte.
    let shape: AnimationShape = AnimationShape::from((reader.read_u32::<T>()? & 0xff) as u8);

    Ok(Self {
      value,
      time,
      shape,
      interpolation: AnimationInterpolation::read_wide::<T, D>(reader)?,
    })
  }

  /// Writes a key the narrow way, `st_Key::Save`, which leaves out the curve parameters of a stepped key.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_f32::<T>(self.value)?;
    writer.write_f32::<T>(self.time)?;
    writer.write_u8(self.shape.into())?;

    if !self.shape.is_stepped() {
      self.interpolation.write::<T>(writer)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;

  use crate::{AnimationInterpolation, AnimationKey, AnimationShape};

  fn new_interpolated_key_bytes() -> Vec<u8> {
    vec![
      0, 0, 192, 63, // Value: 1.5.
      0, 0, 0, 64, // Time: 2.0 seconds.
      0,  // TCB shape.
      0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0,
    ]
  }

  fn new_interpolation() -> AnimationInterpolation {
    AnimationInterpolation {
      tension: -32.0,
      continuity: 32.0,
      bias: -32.0,
      parameters: [32.0, -32.0, 32.0, -32.0],
    }
  }

  #[test]
  fn test_read_write_stepped() -> XrfResult {
    let original: AnimationKey = AnimationKey {
      value: 1.5,
      time: 2.0,
      shape: AnimationShape::Stepped,
      interpolation: AnimationInterpolation::default(),
    };
    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 9);
    assert_eq!(writer.buffer, [0, 0, 192, 63, 0, 0, 0, 64, 4]);

    let mut reader = ChunkReader::from_vec(writer.buffer)?;
    let key: AnimationKey = AnimationKey::read::<XRayByteOrder, _>(&mut reader)?;

    assert_eq!(key, original);
    reader.assert_read("Expect the stepped key to end after its shape")?;

    Ok(())
  }

  #[test]
  fn test_write_stepped_leaves_out_its_parameters() -> XrfResult {
    let original: AnimationKey = AnimationKey {
      value: 1.5,
      time: 2.0,
      shape: AnimationShape::Stepped,
      interpolation: new_interpolation(),
    };
    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.buffer, [0, 0, 192, 63, 0, 0, 0, 64, 4]);

    let mut reader = ChunkReader::from_vec(writer.buffer)?;

    assert_eq!(
      AnimationKey::read::<XRayByteOrder, _>(&mut reader)?,
      AnimationKey {
        interpolation: AnimationInterpolation::default(),
        ..original
      }
    );
    reader.assert_read("Expect the stepped key to end after its shape")?;

    Ok(())
  }

  #[test]
  fn test_read_write_interpolated() -> XrfResult {
    for shape in [0, 1, 2, 3, 5, 7] {
      let original: AnimationKey = AnimationKey {
        value: 1.5,
        time: 2.0,
        shape: AnimationShape::from(shape),
        interpolation: new_interpolation(),
      };
      let mut writer: ChunkWriter = ChunkWriter::new();
      let mut expected: Vec<u8> = new_interpolated_key_bytes();

      expected[8] = shape;
      original.write::<XRayByteOrder>(&mut writer)?;

      assert_eq!(writer.bytes_written(), 23);
      assert_eq!(writer.buffer, expected);

      let mut reader = ChunkReader::from_vec(writer.buffer)?;
      let key: AnimationKey = AnimationKey::read::<XRayByteOrder, _>(&mut reader)?;

      assert_eq!(key, original);
      reader.assert_read("Expect all interpolated key fields to be read")?;
    }

    Ok(())
  }

  #[test]
  fn test_read_wide_stepped_retains_parameters() -> XrfResult {
    let mut bytes: Vec<u8> = vec![0, 0, 192, 63, 0, 0, 0, 64, 4, 3, 2, 1];

    for parameter in [-1.0_f32, 2.0, -3.0, 4.0, -5.0, 6.0, -7.0] {
      bytes.extend_from_slice(&parameter.to_le_bytes());
    }

    assert_eq!(bytes.len(), 40);

    let mut reader = ChunkReader::from_vec(bytes)?;
    let key: AnimationKey = AnimationKey::read_wide::<XRayByteOrder, _>(&mut reader)?;

    assert_eq!(
      key,
      AnimationKey {
        value: 1.5,
        time: 2.0,
        shape: AnimationShape::Stepped,
        interpolation: AnimationInterpolation {
          tension: -1.0,
          continuity: 2.0,
          bias: -3.0,
          parameters: [4.0, -5.0, 6.0, -7.0],
        },
      }
    );
    reader.assert_read("Expect the wide stepped key to include interpolation parameters")?;

    Ok(())
  }

  #[test]
  fn test_read_truncated_key() -> XrfResult {
    let bytes: Vec<u8> = new_interpolated_key_bytes();

    for length in 1..bytes.len() {
      let mut reader = ChunkReader::from_vec(bytes[..length].to_vec())?;

      assert!(AnimationKey::read::<XRayByteOrder, _>(&mut reader).is_err());
    }

    for length in 1..40 {
      let mut reader = ChunkReader::from_vec(vec![0; length])?;

      assert!(AnimationKey::read_wide::<XRayByteOrder, _>(&mut reader).is_err());
    }

    Ok(())
  }
}
