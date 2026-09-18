use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::thm_format::ThmFormat;
use crate::thm_mip_filter::ThmMipFilter;
use crate::thm_texture_flags::ThmTextureFlags;

/// Conversion parameters of a texture, `THM_CHUNK_TEXTUREPARAM` in the engine (`ETextureParams.cpp:68`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThmTextureParamChunk {
  pub format: ThmFormat,
  pub flags: ThmTextureFlags,
  pub border_color: u32,
  pub fade_color: u32,
  pub fade_amount: u32,
  pub mip_filter: ThmMipFilter,
  pub width: u32,
  pub height: u32,
}

impl ThmTextureParamChunk {
  pub const CHUNK_ID: u32 = 0x0812;
}

impl Default for ThmTextureParamChunk {
  /// The values `STextureParams::STextureParams` starts a new descriptor at (`ETextureParams.h`).
  fn default() -> Self {
    Self {
      format: ThmFormat::default(),
      flags: ThmTextureFlags::DEFAULT,
      border_color: 0,
      fade_color: 0,
      fade_amount: 0,
      mip_filter: ThmMipFilter::default(),
      width: 0,
      height: 0,
    }
  }
}

impl ChunkReadWrite for ThmTextureParamChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      format: ThmFormat::from(reader.read_u32::<T>()?),
      flags: ThmTextureFlags::from(reader.read_u32::<T>()?),
      border_color: reader.read_u32::<T>()?,
      fade_color: reader.read_u32::<T>()?,
      fade_amount: reader.read_u32::<T>()?,
      mip_filter: ThmMipFilter::from(reader.read_u32::<T>()?),
      width: reader.read_u32::<T>()?,
      height: reader.read_u32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.format.into())?;
    writer.write_u32::<T>(self.flags.raw())?;
    writer.write_u32::<T>(self.border_color)?;
    writer.write_u32::<T>(self.fade_color)?;
    writer.write_u32::<T>(self.fade_amount)?;
    writer.write_u32::<T>(self.mip_filter.into())?;
    writer.write_u32::<T>(self.width)?;
    writer.write_u32::<T>(self.height)?;

    Ok(())
  }
}
