use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

/// Link to a sliding window table stored outside the visual, `OGF_SWICONTAINER` (chunk 20).
#[derive(Clone, Debug, PartialEq)]
pub struct OgfSwiContainer {
  pub ext_swib_index: u32,
}

impl ChunkReadWrite for OgfSwiContainer {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let container: Self = Self {
      ext_swib_index: reader.read_u32::<T>()?,
    };

    reader.assert_read("Expect all data to be read from ogf swi container")?;

    Ok(container)
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.ext_swib_index)?;

    Ok(())
  }
}
