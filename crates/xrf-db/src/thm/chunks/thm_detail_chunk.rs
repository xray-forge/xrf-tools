use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// Detail texture association of a texture, `THM_CHUNK_DETAIL_EXT` in the engine (`ETextureParams.h`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThmDetailChunk {
  /// Detail texture path without extension, engine-style with backslashes. Empty when none is authored.
  pub name: String,
  pub scale: f32,
}

impl ThmDetailChunk {
  pub const CHUNK_ID: u32 = 0x0815;
}

impl ChunkReadWrite for ThmDetailChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunk: Self = Self {
      name: reader.read_w1251_string()?,
      scale: reader.read_f32::<T>()?,
    };

    reader.assert_read("Expect all data to be read from thm detail chunk")?;

    Ok(chunk)
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_string(&self.name)?;
    writer.write_f32::<T>(self.scale)?;

    Ok(())
  }
}
