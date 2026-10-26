use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkIterator, ChunkReader};
use xrf_error::{XrfError, XrfResult};

use crate::OgfFile;

/// Child visuals carried whole, `OGF_CHILDREN`: numbered chunks from zero, each a visual of its own.
///
/// Read-only, as [`OgfFile`] is: a child is an inspection view, and a narrow edit rewrites the bytes around it instead
/// (see the reference patchers).
#[derive(Debug)]
pub struct OgfChildrenChunk {
  pub nested: Vec<OgfFile>,
}

impl OgfChildrenChunk {
  pub const CHUNK_ID: u32 = 9;

  /// Reads every child from a reader positioned at the children chunk.
  ///
  /// # Errors
  ///
  /// Returns an error when a child is not numbered by its position, or does not read as a visual.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    log::debug!("Reading children chunk: {} bytes", reader.read_bytes_remain());

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
}
