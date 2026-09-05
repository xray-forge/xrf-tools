use std::io::Write;

use serde::{Deserialize, Serialize};
use xrf_chunk::ChunkWriter;
use xrf_error::XrfResult;

/// A chunk the reader did not fold into a field of the descriptor.
///
/// Two things end up here: an id the format has no name for, and a second copy of one it does, since the engine's
/// `find_chunk` stops at the first match and every later copy is data no loader reads.
///
/// They are kept so that rewriting a descriptor cannot quietly destroy what somebody else put in it. A rewrite emits
/// them after the chunks it does know, which is the one way in which a file carrying them does not round trip byte for
/// byte. No descriptor of the workspace corpus carries any: all 13,063 hold the nine known ids, eleven of them with
/// the thumbnail as a tenth.
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
