use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::core::session::SessionId;

/// Directory extraction request for whichever subject the explorer has open.
///
/// One shape for both: the two differ in where the bytes come from, never in what a person asked for.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ArchivesExtractRequest {
  pub session_id: SessionId,
  /// Directory inside the opened tree to extract. An empty prefix means everything.
  pub prefix: String,
  /// Directory to write the contents into.
  pub destination: PathBuf,
}
