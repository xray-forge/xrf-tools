use std::path::Path;

use serde::Serialize;

/// Where a visual is read from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum VisualSource {
  /// A loose `.ogf` file on disk, named by its filesystem path.
  File { path: String },
  /// An asset of the roots, loose or archived, named by its engine identity.
  Asset { logical_path: String },
}

impl VisualSource {
  pub fn label(&self) -> &str {
    match self {
      Self::File { path } => path,
      Self::Asset { logical_path } => logical_path,
    }
  }

  /// Returns the visual's filesystem path when its source provides one.
  pub fn physical_path(&self) -> Option<&Path> {
    match self {
      Self::File { path } => Some(Path::new(path)),
      Self::Asset { .. } => None,
    }
  }
}
