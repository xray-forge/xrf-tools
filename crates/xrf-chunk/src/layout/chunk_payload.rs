use std::io::Write;

use byteorder::ByteOrder;
use xrf_error::XrfResult;

use crate::writer::chunk_writer::ChunkWriter;

/// One chunk as its id and the payload bytes it stores, without the header: what a container keeps of a child no
/// field of it takes, and what [`crate::ChunkSequence`] holds before putting a child down.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChunkPayload {
  pub id: u32,
  /// The payload exactly as stored.
  pub bytes: Vec<u8>,
}

impl ChunkPayload {
  /// Frames the payload under its id and appends it to `destination`.
  ///
  /// # Errors
  ///
  /// Returns an error when the payload is larger than a chunk can declare.
  pub fn write<T: ByteOrder>(&self, destination: &mut ChunkWriter) -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    writer.write_all(&self.bytes)?;
    writer.flush_chunk_into::<T>(destination, self.id)?;

    Ok(())
  }
}
