use serde::{Deserialize, Serialize};

/// Bytes a chunk holds past the fields the reader folded, which no engine reads, kept so a rewrite is the file it was.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThmTrailingBytes {
  /// The id of the chunk they follow the fields of.
  pub id: u32,
  /// The bytes exactly as the file stores them.
  pub data: Vec<u8>,
}
