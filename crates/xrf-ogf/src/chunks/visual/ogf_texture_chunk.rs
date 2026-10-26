use std::io::Write;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// The texture and shader a visual is dressed with, `OGF_TEXTURE`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OgfTextureChunk {
  pub texture_name: String,
  pub shader_name: String,
  /// Bytes the declared chunk size covers past the two names, which the engine never reads: `dxRender_Visual::Load`
  /// finds the chunk and reads two strings (`FBasicVisual.cpp`). Some exporters pad the chunk with zeros; 31 shipped
  /// visuals carry such padding. Kept so the chunk writes back as it was read.
  pub trailing: Vec<u8>,
}

impl OgfTextureChunk {
  pub const CHUNK_ID: u32 = 2;
}

impl ChunkReadWrite for OgfTextureChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      texture_name: reader.read_w1251_string()?,
      shader_name: reader.read_w1251_string()?,
      trailing: reader.read_remaining()?,
    })
  }

  /// Writes the chunk back exactly as read, trailing bytes included.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_string(&self.texture_name)?;
    writer.write_w1251_string(&self.shader_name)?;
    writer.write_all(&self.trailing)?;

    Ok(())
  }
}
