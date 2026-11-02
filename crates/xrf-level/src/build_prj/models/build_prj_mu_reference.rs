use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Matrix4x4;

/// One placement of a multi-use model, `b_mu_reference` (`communicate.h`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildPrjMuReference {
  /// The model, by index into the multi-use models.
  pub model: u32,
  pub transform: Matrix4x4,
  pub flags: u32,
  pub sector: u16,
  /// The two bytes `#pragma pack(4)` puts before the reserved words, kept so a rewrite gives back what was read.
  pub sector_padding: u16,
  pub reserved: [u32; 8],
}

impl BuildPrjMuReference {
  /// Bytes one placement occupies, padding included.
  pub const SERIALIZED_SIZE: u64 = 4 + Matrix4x4::SERIALIZED_SIZE + 4 + 2 + 2 + 8 * 4;
}

impl ChunkReadWrite for BuildPrjMuReference {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let model: u32 = reader.read_u32::<T>()?;
    let transform: Matrix4x4 = reader.read_xr::<T, _>()?;
    let flags: u32 = reader.read_u32::<T>()?;
    let sector: u16 = reader.read_u16::<T>()?;
    let sector_padding: u16 = reader.read_u16::<T>()?;
    let mut reserved: [u32; 8] = [0; 8];

    for value in &mut reserved {
      *value = reader.read_u32::<T>()?;
    }

    Ok(Self {
      model,
      transform,
      flags,
      sector,
      sector_padding,
      reserved,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.model)?;
    writer.write_xr::<T, _>(&self.transform)?;
    writer.write_u32::<T>(self.flags)?;
    writer.write_u16::<T>(self.sector)?;
    writer.write_u16::<T>(self.sector_padding)?;

    for value in self.reserved {
      writer.write_u32::<T>(value)?;
    }

    Ok(())
  }
}
