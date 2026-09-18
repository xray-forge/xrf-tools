use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// How a material treats sound that reaches it, `SGameMtl::MtlAcoustics`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameMtlAcoustics {
  pub absorption: [f32; 3],
  pub scattering: f32,
  pub transmission: [f32; 3],
}

impl GameMtlAcoustics {
  /// Bytes the record occupies, which the engine reads in one `fs.r` over the struct.
  pub const SERIALIZED_SIZE: u64 = 4 * 7;
}

impl ChunkReadWrite for GameMtlAcoustics {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      absorption: [
        reader.read_f32::<T>()?,
        reader.read_f32::<T>()?,
        reader.read_f32::<T>()?,
      ],
      scattering: reader.read_f32::<T>()?,
      transmission: [
        reader.read_f32::<T>()?,
        reader.read_f32::<T>()?,
        reader.read_f32::<T>()?,
      ],
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for value in self.absorption {
      writer.write_f32::<T>(value)?;
    }

    writer.write_f32::<T>(self.scattering)?;

    for value in self.transmission {
      writer.write_f32::<T>(value)?;
    }

    Ok(())
  }
}
