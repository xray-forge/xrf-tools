use std::io::Write;

use xrf_error::XrfResult;

use crate::writer::chunk_writer::ChunkWriter;

/// One chunk's id and payload, without the header: a child kept as stored, or one a [`crate::ChunkSequence`] holds
/// before writing it.
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
  pub fn write(&self, destination: &mut ChunkWriter) -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();

    writer.write_all(&self.bytes)?;
    writer.flush_chunk_into(destination, self.id)?;

    Ok(())
  }
}
