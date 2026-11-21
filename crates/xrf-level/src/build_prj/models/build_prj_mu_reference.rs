use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
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
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let model: u32 = reader.read_u32()?;
    let transform: Matrix4x4 = reader.read_xr()?;
    let flags: u32 = reader.read_u32()?;
    let sector: u16 = reader.read_u16()?;
    let sector_padding: u16 = reader.read_u16()?;
    let mut reserved: [u32; 8] = [0; 8];

    for value in &mut reserved {
      *value = reader.read_u32()?;
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

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32(self.model)?;
    writer.write_xr(&self.transform)?;
    writer.write_u32(self.flags)?;
    writer.write_u16(self.sector)?;
    writer.write_u16(self.sector_padding)?;

    for value in self.reserved {
      writer.write_u32(value)?;
    }

    Ok(())
  }
}
