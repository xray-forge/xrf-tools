use std::io::Write;

use byteorder::ByteOrder;
use xrf_chunk::ChunkWriter;
use xrf_error::XrfResult;

use crate::EditableExtraChunk;

/// The children of one container, encoded in the order `Save` writes them, then put down in the order a file stored
/// them where it is known. The SDK finds every chunk by id, so the order means nothing to it; keeping it is what lets a
/// file another exporter laid out differently come back byte for byte.
pub(crate) struct EditableChunkSequence {
  chunks: Vec<EditableExtraChunk>,
}

impl EditableChunkSequence {
  pub(crate) fn new() -> Self {
    Self { chunks: Vec::new() }
  }

  /// Encodes one child.
  pub(crate) fn push(&mut self, id: u32, write: impl FnOnce(&mut ChunkWriter) -> XrfResult) -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    write(&mut writer)?;

    self.chunks.push(EditableExtraChunk {
      id,
      bytes: writer.buffer,
    });

    Ok(())
  }

  /// Appends the children kept as stored.
  pub(crate) fn push_extra(&mut self, extra: &[EditableExtraChunk]) {
    self.chunks.extend_from_slice(extra);
  }

  /// Puts every child down: first in `order`, each id taking the first child with that id not yet written, then the
  /// rest in the order they were pushed. A container read from a file passes the ids it stored; one built in code
  /// passes none and is written in `Save`'s order.
  pub(crate) fn write_into<T: ByteOrder>(self, destination: &mut ChunkWriter, order: &[u32]) -> XrfResult {
    let mut is_written: Vec<bool> = vec![false; self.chunks.len()];

    for id in order {
      let next: Option<usize> = self
        .chunks
        .iter()
        .enumerate()
        .position(|(index, chunk)| !is_written[index] && chunk.id == *id);

      if let Some(index) = next {
        Self::write_one::<T>(destination, &self.chunks[index])?;
        is_written[index] = true;
      }
    }

    for (index, chunk) in self.chunks.iter().enumerate() {
      if !is_written[index] {
        Self::write_one::<T>(destination, chunk)?;
      }
    }

    Ok(())
  }

  fn write_one<T: ByteOrder>(destination: &mut ChunkWriter, chunk: &EditableExtraChunk) -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    writer.write_all(&chunk.bytes)?;
    writer.flush_chunk_into::<T>(destination, chunk.id)?;

    Ok(())
  }
}
