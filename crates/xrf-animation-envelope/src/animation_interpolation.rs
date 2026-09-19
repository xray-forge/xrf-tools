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

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;

  use crate::AnimationInterpolation;

  #[test]
  fn test_read_write() -> XrfResult {
    let original: AnimationInterpolation = AnimationInterpolation {
      tension: -32.0,
      continuity: 32.0,
      bias: -32.0,
      parameters: [32.0, -32.0, 32.0, -32.0],
    };
    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 14);
    assert_eq!(writer.buffer, [0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0]);

    let mut reader = ChunkReader::from_vec(writer.buffer)?;

    assert_eq!(AnimationInterpolation::read::<XRayByteOrder, _>(&mut reader)?, original);
    reader.assert_read("Expect all interpolation parameters to be read")?;

    Ok(())
  }

  #[test]
  fn test_quantized_bytes_round_trip() -> XrfResult {
    let bytes: Vec<u8> = vec![0, 0, 1, 0, 255, 127, 0, 128, 254, 255, 255, 255, 0, 64];
    let mut reader = ChunkReader::from_vec(bytes.clone())?;
    let interpolation: AnimationInterpolation = AnimationInterpolation::read::<XRayByteOrder, _>(&mut reader)?;
    let values: [f32; 7] = [
      interpolation.tension,
      interpolation.continuity,
      interpolation.bias,
      interpolation.parameters[0],
      interpolation.parameters[1],
      interpolation.parameters[2],
      interpolation.parameters[3],
    ];
    let expected: [f32; 7] = [-32.0, -31.999_023, -0.000_488, 0.000_488, 31.999_023, 32.0, -15.999_756];

    for (value, expected) in values.into_iter().zip(expected) {
      assert!((value - expected).abs() < 0.000_01, "Expected {expected}, got {value}");
    }

    reader.assert_read("Expect all quantized parameters to be read")?;

    let mut writer: ChunkWriter = ChunkWriter::new();

    interpolation.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.buffer, bytes);

    Ok(())
  }

  #[test]
  fn test_read_wide() -> XrfResult {
    let bytes: Vec<u8> = [-1.25_f32, 2.5, -3.75, 4.0, -5.5, 6.25, 33.5]
      .into_iter()
      .flat_map(f32::to_le_bytes)
      .collect();
    let mut reader = ChunkReader::from_vec(bytes)?;

    assert_eq!(
      AnimationInterpolation::read_wide::<XRayByteOrder, _>(&mut reader)?,
      AnimationInterpolation {
        tension: -1.25,
        continuity: 2.5,
        bias: -3.75,
        parameters: [4.0, -5.5, 6.25, 33.5],
      }
    );
    reader.assert_read("Expect all wide interpolation parameters to be read")?;

    Ok(())
  }

  #[test]
  fn test_read_truncated_parameters() -> XrfResult {
    for length in 1..14 {
      let mut reader = ChunkReader::from_vec(vec![0; length])?;

      assert!(AnimationInterpolation::read::<XRayByteOrder, _>(&mut reader).is_err());
    }

    for length in 1..28 {
      let mut reader = ChunkReader::from_vec(vec![0; length])?;

      assert!(AnimationInterpolation::read_wide::<XRayByteOrder, _>(&mut reader).is_err());
    }

    Ok(())
  }
}
