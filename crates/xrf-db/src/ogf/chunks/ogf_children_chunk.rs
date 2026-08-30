use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkIterator, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::to_format_size;

use crate::OgfFile;

#[derive(Debug)]
pub struct OgfChildrenChunk {
  pub nested: Vec<OgfFile>,
}

impl OgfChildrenChunk {
  pub const CHUNK_ID: u32 = 9;
}

impl ChunkReadWrite for OgfChildrenChunk {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    log::info!("Reading children chunk: {} bytes", reader.read_bytes_remain());

    let mut children: Vec<OgfFile> = Vec::new();

    for (index, object_reader) in (0..).zip(ChunkIterator::from_start(reader)?) {
      let mut object_reader: ChunkReader<D> = object_reader?;

      if object_reader.id != index {
        return Err(XrfError::new_unexpected_error(format!(
          "Invalid data in OGF children chunk, expected index {}, got {}",
          index, object_reader.id
        )));
      }

      children.push(OgfFile::read_from_chunk::<T, _>(&mut object_reader)?);
    }

    reader.assert_read("Expect all data to be read from ogf children")?;

    Ok(Self { nested: children })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    for (index, _child) in self.nested.iter().enumerate() {
      let mut child_writer: ChunkWriter = ChunkWriter::new();

      // todo: Child write.

      child_writer.flush_chunk_into::<T>(writer, to_format_size(index, "ogf child chunk id")?)?;
    }

    todo!("Implement OGF file writer here");
  }
}
