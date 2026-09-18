use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// ogf_desc c++ class
#[derive(Debug)]
pub struct OgfDescriptionChunk {
  pub source_file: String,
  pub convertor: String,
  pub built_at: u32,
  pub creator: String,
  pub created_at: u32,
  pub editor: String,
  pub edited_at: u32,
}

impl OgfDescriptionChunk {
  pub const CHUNK_ID: u32 = 18;
}

impl ChunkReadWrite for OgfDescriptionChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let description: Self = Self {
      source_file: reader.read_w1251_string()?,
      convertor: reader.read_w1251_string()?,
      built_at: reader.read_u32::<T>()?,
      creator: reader.read_w1251_string()?,
      created_at: reader.read_u32::<T>()?,
      editor: reader.read_w1251_string()?,
      edited_at: reader.read_u32::<T>()?,
    };

    reader.assert_read("Expect all data to be read from ogf description")?;

    Ok(description)
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_string(&self.source_file)?;
    writer.write_w1251_string(&self.convertor)?;
    writer.write_u32::<T>(self.built_at)?;
    writer.write_w1251_string(&self.creator)?;
    writer.write_u32::<T>(self.created_at)?;
    writer.write_w1251_string(&self.editor)?;
    writer.write_u32::<T>(self.edited_at)?;

    Ok(())
  }
}
