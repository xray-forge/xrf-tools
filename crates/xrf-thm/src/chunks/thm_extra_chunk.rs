use serde::{Deserialize, Serialize};

/// A chunk the reader did not fold into a field of the descriptor, written back framed under its id as it was read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThmExtraChunk {
  /// The id the chunk was found under, written back unchanged.
  pub id: u32,
  /// The payload exactly as the file stores it.
  pub data: Vec<u8>,
}
