use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// Conversion parameters of a texture, `THM_CHUNK_TEXTUREPARAM` in the engine (`ETextureParams.cpp`).
///
/// Almost all of it is authoring data the converter consumes and the runtime ignores. The exception is two bits of
/// `flags`: `flDiffuseDetail` and `flBumpDetail` decide whether the detail chunk is applied at all and how
/// (`TextureDescrManager.cpp`), which is why this chunk is read.
///
/// The fields are read in the engine's order and the chunk is not required to end there: `STextureParams::Load`
/// reads these eight and nothing after, so trailing bytes an authoring tool appended cost nothing in the engine and
/// cost nothing here.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThmTextureParamChunk {
  pub format: u32,
  pub flags: u32,
  pub border_color: u32,
  pub fade_color: u32,
  pub fade_amount: u32,
  pub mip_filter: u32,
  pub width: u32,
  pub height: u32,
}

impl ThmTextureParamChunk {
  pub const CHUNK_ID: u32 = 0x0812;

  /// `STextureParams` flags, `ETextureParams.h:82`.
  pub const FLAG_DIFFUSE_DETAIL: u32 = 1 << 23;
  pub const FLAG_BUMP_DETAIL: u32 = 1 << 26;

  pub fn is_diffuse_detail(&self) -> bool {
    self.flags & Self::FLAG_DIFFUSE_DETAIL != 0
  }

  pub fn is_bump_detail(&self) -> bool {
    self.flags & Self::FLAG_BUMP_DETAIL != 0
  }
}

impl ChunkReadWrite for ThmTextureParamChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      format: reader.read_u32::<T>()?,
      flags: reader.read_u32::<T>()?,
      border_color: reader.read_u32::<T>()?,
      fade_color: reader.read_u32::<T>()?,
      fade_amount: reader.read_u32::<T>()?,
      mip_filter: reader.read_u32::<T>()?,
      width: reader.read_u32::<T>()?,
      height: reader.read_u32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.format)?;
    writer.write_u32::<T>(self.flags)?;
    writer.write_u32::<T>(self.border_color)?;
    writer.write_u32::<T>(self.fade_color)?;
    writer.write_u32::<T>(self.fade_amount)?;
    writer.write_u32::<T>(self.mip_filter)?;
    writer.write_u32::<T>(self.width)?;
    writer.write_u32::<T>(self.height)?;

    Ok(())
  }
}
