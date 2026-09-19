use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::AnimationInterpolation;

/// One key of an animation envelope: a value at a time, and the curve leading to it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationKey {
  pub value: f32,
  /// Seconds from the start of the animation.
  pub time: f32,
  pub shape: u8,
  pub interpolation: Option<AnimationInterpolation>,
}

impl AnimationKey {
  /// Shape value that means the key steps rather than interpolates, `st_Key::shape` of 4.
  const SHAPE_STEPPED: u8 = 4;

  /// Bytes a stepped key occupies: the value, the time and the shape.
  pub const MIN_SERIALIZED_SIZE: u64 = 4 + 4 + 1;

  /// Bytes a wide key occupies, including the seven unquantized interpolation parameters.
  pub(crate) const WIDE_SERIALIZED_SIZE: u64 = 4 + 4 + 4 + 7 * 4;

  /// Whether this key steps to its value rather than interpolating towards it.
  pub const fn is_stepped(&self) -> bool {
    self.shape == Self::SHAPE_STEPPED
  }

  /// Reads a key the narrow way, `st_Key::Load_2`.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload ends inside the key.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let value: f32 = reader.read_f32::<T>()?;
    let time: f32 = reader.read_f32::<T>()?;
    let shape: u8 = reader.read_u8()?;

    Ok(Self {
      value,
      time,
      shape,
      interpolation: if shape == Self::SHAPE_STEPPED {
        None
      } else {
        Some(AnimationInterpolation::read::<T, D>(reader)?)
      },
    })
  }

  /// Reads a key the wide way, `st_Key::Load_1`.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload ends inside the key.
  pub fn read_wide<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let value: f32 = reader.read_f32::<T>()?;
    let time: f32 = reader.read_f32::<T>()?;
    let shape: u32 = reader.read_u32::<T>()?;

    Ok(Self {
      value,
      time,
      // Narrowed the way the engine does, which keeps only the low byte.
      shape: (shape & 0xff) as u8,
      interpolation: Some(AnimationInterpolation::read_wide::<T, D>(reader)?),
    })
  }

  /// Writes a key the narrow way, `st_Key::Save`.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_f32::<T>(self.value)?;
    writer.write_f32::<T>(self.time)?;
    writer.write_u8(self.shape)?;

    if let Some(interpolation) = &self.interpolation {
      interpolation.write::<T>(writer)?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;

  use crate::{AnimationInterpolation, AnimationKey};

  fn new_interpolated_key_bytes() -> Vec<u8> {
    vec![
      0, 0, 192, 63, // Value: 1.5.
      0, 0, 0, 64, // Time: 2.0 seconds.
      0,  // TCB shape.
      0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0,
    ]
  }

  #[test]
  fn test_read_write_stepped() -> XrfResult {
    let original: AnimationKey = AnimationKey {
      value: 1.5,
      time: 2.0,
      shape: 4,
      interpolation: None,
    };
    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 9);
    assert_eq!(writer.buffer, [0, 0, 192, 63, 0, 0, 0, 64, 4]);

    let mut reader = ChunkReader::from_vec(writer.buffer)?;
    let key: AnimationKey = AnimationKey::read::<XRayByteOrder, _>(&mut reader)?;

    assert_eq!(key, original);
    assert!(key.is_stepped());
    reader.assert_read("Expect the stepped key to end after its shape")?;

    Ok(())
  }

  #[test]
  fn test_read_write_interpolated() -> XrfResult {
    for shape in [0, 1, 2, 3, 5] {
      let original: AnimationKey = AnimationKey {
        value: 1.5,
        time: 2.0,
        shape,
        interpolation: Some(AnimationInterpolation {
          tension: -32.0,
          continuity: 32.0,
          bias: -32.0,
          parameters: [32.0, -32.0, 32.0, -32.0],
        }),
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
      assert!(!key.is_stepped());
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
        shape: 4,
        interpolation: Some(AnimationInterpolation {
          tension: -1.0,
          continuity: 2.0,
          bias: -3.0,
          parameters: [4.0, -5.0, 6.0, -7.0],
        }),
      }
    );
    assert!(key.is_stepped());
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
