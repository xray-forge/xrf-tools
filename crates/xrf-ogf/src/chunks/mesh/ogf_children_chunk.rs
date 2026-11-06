use std::io::Write;

use byteorder::ByteOrder;
use xrf_chunk::{ChunkDataSource, ChunkIterator, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};

use crate::OgfFile;

/// Child visuals carried whole, `OGF_CHILDREN`: numbered chunks from zero, each a visual of its own.
///
/// Read as [`OgfFile`]s, which are inspection views: a narrow edit rewrites the bytes around a child instead (see the
/// reference patchers). A new visual's children are written from their own bytes, [`Self::write_nested`].
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

  /// Writes the children chunk around `children`, each a child visual's chunks, numbered from zero in order.
  ///
  /// # Errors
  ///
  /// Returns an error when there are more children than a `u32` numbers, or the writer refuses the bytes.
  pub fn write_nested<T: ByteOrder>(writer: &mut ChunkWriter, children: &[Vec<u8>]) -> XrfResult {
    let mut nested: ChunkWriter = ChunkWriter::new();

    for (index, child) in children.iter().enumerate() {
      let mut numbered: ChunkWriter = ChunkWriter::new();

      numbered.write_all(child)?;
      numbered.flush_chunk_into::<T>(
        &mut nested,
        u32::try_from(index)
          .map_err(|_| XrfError::new_invalid_error("An OGF holds more children than a u32 numbers"))?,
      )?;
    }

    nested.flush_chunk_into::<T>(writer, Self::CHUNK_ID)?;

    Ok(())
  }
}
