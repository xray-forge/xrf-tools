use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Archive unpacking request.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ArchivesUnpackRequest {
  /// Archive or directory of archives to read.
  pub from: PathBuf,
  /// Directory to write the contents into.
  pub destination: PathBuf,
}
