use std::io;
use std::io::Write;

use byteorder::{ByteOrder, WriteBytesExt};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::to_format_size;

#[derive(Default)]
pub struct ChunkWriter {
  pub buffer: Vec<u8>,
}

impl ChunkWriter {
  /// Creates an empty chunk payload buffer.
  pub fn new() -> Self {
    Self::default()
  }

  /// Writes the buffered payload as an X-Ray chunk to `destination` without clearing the buffer.
  ///
  /// The header contains `id` and the payload length in the selected byte order.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload exceeds the format's `u32` length limit or the destination rejects a write.
  pub fn flush_chunk_into<T: ByteOrder>(&mut self, destination: &mut dyn Write, id: u32) -> XrfResult<usize> {
    let payload_size: u32 = to_format_size(self.buffer.len(), "Chunk payload")?;

    destination.write_u32::<T>(id)?;
    destination.write_u32::<T>(payload_size)?;
    destination.write_all(&self.buffer)?;

    Ok(self.buffer.len())
  }

  /// Writes the buffered payload without a chunk header, leaving the buffer unchanged.
  pub fn flush_raw_into(&mut self, file: &mut dyn Write) -> XrfResult {
    Ok(file.write_all(&self.buffer)?)
  }

  /// Returns the buffered payload framed as an X-Ray chunk, without clearing the buffer.
  ///
  /// # Errors
  ///
  /// Returns an error when the framed size exceeds platform capacity or the payload exceeds the format's `u32` limit.
  pub fn flush_chunk_into_buffer<T: ByteOrder>(&mut self, id: u32) -> XrfResult<Vec<u8>> {
    let capacity: usize = self
      .buffer
      .len()
      .checked_add(size_of::<u32>() * 2)
      .ok_or_else(|| XrfError::new_invalid_error("Framed chunk size exceeds the platform limit"))?;

    let mut buffer: Vec<u8> = Vec::with_capacity(capacity);

    let payload_size: u32 = to_format_size(self.buffer.len(), "Chunk payload")?;

    buffer.write_u32::<T>(id)?;
    buffer.write_u32::<T>(payload_size)?;
    buffer.write_all(&self.buffer)?;

    Ok(buffer)
  }

  /// Returns a copy of the buffered payload without a chunk header.
  pub fn flush_raw_into_buffer(&mut self) -> XrfResult<Vec<u8>> {
    let mut buffer: Vec<u8> = Vec::with_capacity(self.buffer.len());

    buffer.write_all(&self.buffer)?;

    Ok(buffer)
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
  use crate::XRayByteOrder;

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

    assert_eq!(chunk.flush_chunk_into::<XRayByteOrder>(&mut destination, 7)?, 4);
    assert_eq!(destination.bytes, [7, 0, 0, 0, 4, 0, 0, 0, 1, 2, 3, 4]);

    Ok(())
  }
}
