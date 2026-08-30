use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

#[derive(Debug)]
pub struct OgfTextureChunk {
  pub texture_name: String,
  pub shader_name: String,
}

impl OgfTextureChunk {
  pub const CHUNK_ID: u32 = 2;
}

impl ChunkReadWrite for OgfTextureChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let texture: Self = Self {
      texture_name: reader.read_w1251_string()?,
      shader_name: reader.read_w1251_string()?,
    };

    reader.assert_read("Expect all data to be read from ogf texture")?;

    Ok(texture)
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_string(&self.texture_name)?;
    writer.write_w1251_string(&self.shader_name)?;

    Ok(())
  }
}
