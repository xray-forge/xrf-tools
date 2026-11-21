use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;

use crate::texture::thm_format::ThmFormat;
use crate::texture::thm_mip_filter::ThmMipFilter;
use crate::texture::thm_texture_flags::ThmTextureFlags;
use crate::texture::thm_texture_type::ThmTextureType;

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

  /// What the SDK's editor changes when the descriptor is given `texture_type`, `STextureParams::OnTypeChange`: the
  /// flags it clears, then the ones it sets, then the format and mip filter where the type has its own.
  pub fn apply_texture_type(&mut self, texture_type: ThmTextureType) {
    let cleared: u32 = texture_type.get_cleared_flags().raw();

    self.flags = ThmTextureFlags::from((self.flags.raw() & !cleared) | texture_type.get_set_flags().raw());

    if let Some(format) = texture_type.get_format() {
      self.format = format;
    }

    if let Some(mip_filter) = texture_type.get_mip_filter() {
      self.mip_filter = mip_filter;
    }
  }
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
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      format: ThmFormat::from(reader.read_u32()?),
      flags: ThmTextureFlags::from(reader.read_u32()?),
      border_color: reader.read_u32()?,
      fade_color: reader.read_u32()?,
      fade_amount: reader.read_u32()?,
      mip_filter: ThmMipFilter::from(reader.read_u32()?),
      width: reader.read_u32()?,
      height: reader.read_u32()?,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32(self.format.into())?;
    writer.write_u32(self.flags.raw())?;
    writer.write_u32(self.border_color)?;
    writer.write_u32(self.fade_color)?;
    writer.write_u32(self.fade_amount)?;
    writer.write_u32(self.mip_filter.into())?;
    writer.write_u32(self.width)?;
    writer.write_u32(self.height)?;

    Ok(())
  }
}
