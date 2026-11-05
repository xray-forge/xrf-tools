use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// Who made an editable object and when, `EOBJ_CHUNK_DESC`. The times are 32-bit Unix seconds, which is what the
/// SDK's `time_t` was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditableDescription {
  pub creator: String,
  pub created_at: u32,
  pub modifier: String,
  pub modified_at: u32,
}

impl ChunkReadWrite for EditableDescription {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      creator: reader.read_w1251_string()?,
      created_at: reader.read_u32::<T>()?,
      modifier: reader.read_w1251_string()?,
      modified_at: reader.read_u32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_w1251_string(&self.creator)?;
    writer.write_u32::<T>(self.created_at)?;
    writer.write_w1251_string(&self.modifier)?;
    writer.write_u32::<T>(self.modified_at)?;

    Ok(())
  }
}
