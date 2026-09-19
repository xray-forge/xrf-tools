use byteorder::{ByteOrder, ReadBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// The curve parameters a key that interpolates carries.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationInterpolation {
  pub tension: f32,
  pub continuity: f32,
  pub bias: f32,
  pub parameters: [f32; 4],
}

impl AnimationInterpolation {
  /// Range the interpolation parameters of a key are quantised across, from `st_Key::Load_2`.
  const PARAMETER_RANGE: (f32, f32) = (-32.0, 32.0);

  /// Bytes the quantised form occupies: seven `u16`.
  pub const SERIALIZED_SIZE: u64 = 7 * 2;

  pub(crate) fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let (minimum, maximum): (f32, f32) = Self::PARAMETER_RANGE;

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

  pub(crate) fn read_wide<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
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

  pub(crate) fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    let (minimum, maximum): (f32, f32) = Self::PARAMETER_RANGE;

    writer.write_f32_q16::<T>(self.tension, minimum, maximum)?;
    writer.write_f32_q16::<T>(self.continuity, minimum, maximum)?;
    writer.write_f32_q16::<T>(self.bias, minimum, maximum)?;

    for parameter in self.parameters {
      writer.write_f32_q16::<T>(parameter, minimum, maximum)?;
    }

    Ok(())
  }
}
