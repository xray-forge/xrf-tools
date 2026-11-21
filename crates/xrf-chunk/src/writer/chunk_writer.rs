use std::io;
use std::io::Write;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::narrow_format_size;

use crate::bytes::xray_write::XRayWrite;
use crate::chunk_constants::CHUNK_HEADER_SIZE;

/// A chunk payload being built, framed with its header only when flushed.
///
/// A child is built in its own writer and flushed into its parent ([`Self::flush_chunk_into`]); the root's bytes are
/// taken with [`Self::into_bytes`]. Flushing leaves the buffer as it was; clear it or create another writer before
/// building an unrelated payload.
#[derive(Default)]
pub struct ChunkWriter {
  pub buffer: Vec<u8>,
}

impl ChunkWriter {
  /// Creates an empty chunk payload buffer.
  pub fn new() -> Self {
    Self::default()
  }

  /// Writes `payload` to `destination` as an X-Ray chunk under `id`: the id and the payload length, little-endian, then
  /// the payload. Returns the payload length.
  ///
  /// The one way a chunk is framed, for bytes already held as well as for a writer's buffer
  /// ([`Self::flush_chunk_into`]), so a payload is never copied into a writer only to be framed.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload exceeds the format's `u32` length limit or the destination rejects a write.
  pub fn write_framed(destination: &mut dyn Write, id: u32, payload: &[u8]) -> XrfResult<usize> {
    let payload_size: u32 = narrow_format_size(payload.len(), "Chunk payload")?;

    destination.write_u32(id)?;
    destination.write_u32(payload_size)?;
    destination.write_all(payload)?;

    Ok(payload.len())
  }

  /// Writes the buffered payload as an X-Ray chunk to `destination`, which is how a child is placed in its parent:
  /// `child.flush_chunk_into(&mut parent, id)`. See [`Self::write_framed`].
  ///
  /// # Errors
  ///
  /// Returns an error when the payload exceeds the format's `u32` length limit or the destination rejects a write.
  pub fn flush_chunk_into(&self, destination: &mut dyn Write, id: u32) -> XrfResult<usize> {
    Self::write_framed(destination, id, &self.buffer)
  }

  /// Writes the buffered payload to `destination` without a chunk header.
  pub fn flush_raw_into(&self, destination: &mut dyn Write) -> XrfResult {
    Ok(destination.write_all(&self.buffer)?)
  }

  /// The buffered payload, without a chunk header or a copy: the end of a writer whose bytes are the value wanted.
  pub fn into_bytes(self) -> Vec<u8> {
    self.buffer
  }

  /// Returns the buffered payload framed as an X-Ray chunk, for a caller that wants the chunk as a value.
  ///
  /// Not how a child joins its parent: [`Self::flush_chunk_into`] writes it there without the intermediate copy.
  ///
  /// # Errors
  ///
  /// Returns an error when the framed size exceeds platform capacity or the payload exceeds the format's `u32` limit.
  pub fn flush_chunk_into_buffer(&self, id: u32) -> XrfResult<Vec<u8>> {
    let capacity: usize = self
      .buffer
      .len()
      .checked_add(CHUNK_HEADER_SIZE as usize)
      .ok_or_else(|| XrfError::new_invalid_error("Framed chunk size exceeds the platform limit"))?;
    let mut framed: Vec<u8> = Vec::with_capacity(capacity);

    self.flush_chunk_into(&mut framed, id)?;

    Ok(framed)
  }

  /// Returns the current payload length in bytes.
  pub fn bytes_written(&self) -> usize {
    self.buffer.len()
  }
}

impl Write for ChunkWriter {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    self.buffer.write(buf)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.buffer.flush()
  }
}

#[cfg(test)]
mod tests {
  use std::io::{Result as IoResult, Write};

  use xrf_error::XrfResult;

  use super::ChunkWriter;

  struct ShortWriter {
    bytes: Vec<u8>,
    max_write: usize,
  }

  impl Write for ShortWriter {
    fn write(&mut self, buffer: &[u8]) -> IoResult<usize> {
      let written: usize = buffer.len().min(self.max_write);

      self.bytes.extend_from_slice(&buffer[..written]);

      Ok(written)
    }

    fn flush(&mut self) -> IoResult<()> {
      Ok(())
    }
  }

  #[test]
  fn flush_chunk_writes_the_complete_payload_to_short_writers() -> XrfResult {
    let mut chunk: ChunkWriter = ChunkWriter::new();
    let mut destination = ShortWriter {
      bytes: Vec::new(),
      max_write: 2,
    };

    chunk.write_all(&[1, 2, 3, 4])?;

    assert_eq!(chunk.flush_chunk_into(&mut destination, 7)?, 4);
    assert_eq!(destination.bytes, [7, 0, 0, 0, 4, 0, 0, 0, 1, 2, 3, 4]);

    Ok(())
  }

  #[test]
  fn flushing_into_a_buffer_frames_the_payload_and_keeps_it() -> XrfResult {
    let mut chunk: ChunkWriter = ChunkWriter::new();

    chunk.write_all(&[1, 2])?;

    assert_eq!(chunk.flush_chunk_into_buffer(3)?, [3, 0, 0, 0, 2, 0, 0, 0, 1, 2]);
    assert_eq!(chunk.into_bytes(), [1, 2], "the payload survives a flush");

    Ok(())
  }

  #[test]
  fn frames_bytes_it_is_handed_as_it_frames_a_writer() -> XrfResult {
    let mut chunk: ChunkWriter = ChunkWriter::new();
    let mut framed: Vec<u8> = Vec::new();

    chunk.write_all(&[1, 2])?;

    assert_eq!(ChunkWriter::write_framed(&mut framed, 3, &[1, 2])?, 2);
    assert_eq!(framed, chunk.flush_chunk_into_buffer(3)?);

    Ok(())
  }
}
