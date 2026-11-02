use std::path::PathBuf;

use serde::Serialize;
use xrf_utils::format_path;

/// The physical container of a located asset.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
// `rename_all_fields` keeps struct-variant fields camel-cased alongside the variants.
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum XrayAssetContainer {
  /// A loose file, preserving its root so consumers can identify the winning overlay.
  Directory { root: PathBuf, relative_path: PathBuf },
  /// An entry inside the archive volume file at `path`: the one volume holding this copy, not its set's root.
  Archive { path: PathBuf },
}

impl XrayAssetContainer {
  /// Describes where bytes sit, for a reader rather than for a lookup.
  pub fn format(&self) -> String {
    match self {
      Self::Directory { root, .. } => root.display().to_string(),
      Self::Archive { path } => format!("{} (archive)", format_path(path)),
    }
  }
}
