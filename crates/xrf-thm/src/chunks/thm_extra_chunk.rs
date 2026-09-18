use std::io::Write;

use serde::{Deserialize, Serialize};
use xrf_chunk::ChunkWriter;
use xrf_error::XrfResult;

/// A chunk the reader did not fold into a field of the descriptor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThmExtraChunk {
  /// The id the chunk was found under, written back unchanged.
  pub id: u32,
  /// The payload exactly as the file stores it.
  pub data: Vec<u8>,
}

impl ThmExtraChunk {
  /// Writes the payload, which is copied rather than re-serialized.
  pub fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_all(&self.data)?;

    Ok(())
  }
}
