use serde::{Deserialize, Serialize};

/// One `[include_folders]` or `[exclude_folders]` entry.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePackDirectory {
  pub path: String,
  pub is_recursive: bool,
}
