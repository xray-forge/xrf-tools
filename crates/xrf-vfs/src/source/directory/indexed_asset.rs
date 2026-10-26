use std::path::{Path, PathBuf};

/// One entry of a [`crate::source::directory::XrayAssetIndex`], pairing an engine identity with the file it was derived
/// from.
///
/// Borrowed and crate-internal: it exists only to let a directory source answer a lookup without copying, and callers
/// outside the crate get the owned [`crate::XrayAsset`] from the VFS instead.
#[derive(Clone, Copy, Debug)]
pub(crate) struct IndexedAsset<'a> {
  pub(crate) relative_path: &'a Path,
  pub(crate) logical_path: &'a str,
  pub(crate) root: &'a Path,
}

impl IndexedAsset<'_> {
  /// Returns the canonical lower-case, backslash-separated path used by the X-Ray engine.
  ///
  /// Unlike [`Self::relative_path`], this is an engine identity rather than a physical filesystem path.
  pub(crate) fn logical_path(&self) -> &str {
    self.logical_path
  }

  /// Returns this asset's root-relative physical filesystem path.
  pub(crate) fn relative_path(&self) -> &Path {
    self.relative_path
  }

  /// Resolves this asset's physical path from the indexed root.
  pub(crate) fn absolute_path(&self) -> PathBuf {
    self.root.join(self.relative_path)
  }
}
