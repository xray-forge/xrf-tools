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
