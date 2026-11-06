use byteorder::ByteOrder;
use xrf_error::XrfResult;

use crate::layout::chunk_payload::ChunkPayload;
use crate::writer::chunk_writer::ChunkWriter;

/// A container's children being written: encoded one by one in the order its format's own writer puts them, then put
/// down by a [`crate::ChunkLayout`] in the order a file stored them.
#[derive(Debug, Default)]
pub struct ChunkSequence {
  chunks: Vec<ChunkPayload>,
}

impl ChunkSequence {
  pub fn new() -> Self {
    Self::default()
  }

  /// Encodes one child under `id`.
  ///
  /// # Errors
  ///
  /// Returns the error `write` returns.
  pub fn push(&mut self, id: u32, write: impl FnOnce(&mut ChunkWriter) -> XrfResult) -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    write(&mut writer)?;

    self.chunks.push(ChunkPayload {
      id,
      bytes: writer.buffer,
    });

    Ok(())
  }

  /// Appends children kept as stored, after the encoded ones.
  pub(crate) fn push_kept(&mut self, kept: &[ChunkPayload]) {
    self.chunks.extend_from_slice(kept);
  }

  /// Puts every child down: first in `order`, each id taking the first child with that id not yet written, then the
  /// rest in the order they were pushed.
  pub(crate) fn write_in_order<T: ByteOrder>(self, destination: &mut ChunkWriter, order: &[u32]) -> XrfResult {
    let mut is_written: Vec<bool> = vec![false; self.chunks.len()];

    for id in order {
      let next: Option<usize> =
        (0..self.chunks.len()).find(|index| !is_written[*index] && self.chunks[*index].id == *id);

      if let Some(index) = next {
        self.chunks[index].write::<T>(destination)?;
        is_written[index] = true;
      }
    }

    for (index, chunk) in self.chunks.iter().enumerate() {
      if !is_written[index] {
        chunk.write::<T>(destination)?;
      }
    }

    Ok(())
  }
}
