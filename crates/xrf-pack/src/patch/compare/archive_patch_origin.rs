use std::path::PathBuf;

use serde::Serialize;
use xrf_vfs::XrayAssetContainer;

/// One place a comparison read entries from, listed once per report and referred to by index.
///
/// Mirrors [`XrayAssetContainer`] without its `relative_path`. That field is the reason a container cannot simply be
/// shared — it differs per entry — and it is also the reason sharing is worth arranging: nothing downstream reads it,
/// because the logical name is the identity every consumer of a comparison already works in.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
// `rename_all_fields` keeps struct-variant fields camel-cased alongside the variants, as the container it mirrors does.
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ArchivePatchOrigin {
  /// A loose tree, named by the root it mounted at.
  Directory { root: PathBuf },
  /// The archive volume set at `path`.
  Archive { path: PathBuf },
}

impl From<&XrayAssetContainer> for ArchivePatchOrigin {
  fn from(container: &XrayAssetContainer) -> Self {
    match container {
      XrayAssetContainer::Directory { root, .. } => Self::Directory { root: root.clone() },
      XrayAssetContainer::Archive { path } => Self::Archive { path: path.clone() },
    }
  }
}
