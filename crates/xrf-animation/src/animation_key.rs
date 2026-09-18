use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// Range the interpolation parameters of a key are quantised across, from `st_Key::Load_2`.
const PARAMETER_RANGE: (f32, f32) = (-32.0, 32.0);

/// Shape value that means the key steps rather than interpolates, `st_Key::shape` of 4.
const SHAPE_STEPPED: u8 = 4;

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

/// The curve parameters a key that interpolates carries.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationInterpolation {
  pub tension: f32,
  pub continuity: f32,
  pub bias: f32,
  pub parameters: [f32; 4],
}

impl AnimationKey {
  /// Bytes a stepped key occupies: the value, the time and the shape.
  pub const MIN_SERIALIZED_SIZE: u64 = 4 + 4 + 1;

  /// Whether this key steps to its value rather than interpolating towards it.
  pub const fn is_stepped(&self) -> bool {
    self.shape == SHAPE_STEPPED
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
      interpolation: if shape == SHAPE_STEPPED {
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

impl AnimationInterpolation {
  /// Bytes the quantised form occupies: seven `u16`.
  pub const SERIALIZED_SIZE: u64 = 7 * 2;

  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let (minimum, maximum): (f32, f32) = PARAMETER_RANGE;

    Ok(Self {
      tension: reader.read_f32_q16::<T>(minimum, maximum)?,
      continuity: reader.read_f32_q16::<T>(minimum, maximum)?,
      bias: reader.read_f32_q16::<T>(minimum, maximum)?,
      parameters: [
        reader.read_f32_q16::<T>(minimum, maximum)?,
        reader.read_f32_q16::<T>(minimum, maximum)?,
        reader.read_f32_q16::<T>(minimum, maximum)?,
        reader.read_f32_q16::<T>(minimum, maximum)?,
      ],
    })
  }

  fn read_wide<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      tension: reader.read_f32::<T>()?,
      continuity: reader.read_f32::<T>()?,
      bias: reader.read_f32::<T>()?,
      parameters: [
        reader.read_f32::<T>()?,
        reader.read_f32::<T>()?,
        reader.read_f32::<T>()?,
        reader.read_f32::<T>()?,
      ],
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let (minimum, maximum): (f32, f32) = PARAMETER_RANGE;

    writer.write_f32_q16::<T>(self.tension, minimum, maximum)?;
    writer.write_f32_q16::<T>(self.continuity, minimum, maximum)?;
    writer.write_f32_q16::<T>(self.bias, minimum, maximum)?;

    for parameter in self.parameters {
      writer.write_f32_q16::<T>(parameter, minimum, maximum)?;
    }

    Ok(())
  }
}
