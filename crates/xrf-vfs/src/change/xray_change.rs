use xrf_error::XrfResult;

use crate::{XrayChangeKind, XrayLogicalPath, XrayMountId};

/// One logical path a change moved in one mount: what a commit wrote there, or what a watcher saw change on disk.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct XrayChange {
  mount: XrayMountId,
  path: XrayLogicalPath,
  kind: XrayChangeKind,
}

impl XrayChange {
  /// A mount's copy of a logical path now holding other bytes.
  ///
  /// # Errors
  ///
  /// Returns an error when `path` is not a valid X-Ray logical path.
  pub fn modified(mount: XrayMountId, path: &str) -> XrfResult<Self> {
    Self::new(mount, path, XrayChangeKind::Modified)
  }

  /// A logical path a mount did not hold, now on disk under it.
  ///
  /// # Errors
  ///
  /// Returns an error when `path` is not a valid X-Ray logical path.
  pub fn created(mount: XrayMountId, path: &str) -> XrfResult<Self> {
    Self::new(mount, path, XrayChangeKind::Created)
  }

  /// A logical path a mount held, gone from disk.
  ///
  /// # Errors
  ///
  /// Returns an error when `path` is not a valid X-Ray logical path.
  pub fn deleted(mount: XrayMountId, path: &str) -> XrfResult<Self> {
    Self::new(mount, path, XrayChangeKind::Deleted)
  }

  fn new(mount: XrayMountId, path: &str, kind: XrayChangeKind) -> XrfResult<Self> {
    Ok(Self {
      mount,
      path: XrayLogicalPath::new(path)?,
      kind,
    })
  }

  pub fn get_mount(&self) -> XrayMountId {
    self.mount
  }

  pub fn get_path(&self) -> &XrayLogicalPath {
    &self.path
  }

  pub fn get_kind(&self) -> XrayChangeKind {
    self.kind
  }

  /// Whether the change moved which paths its mount holds, rather than only the bytes of one it held.
  pub fn is_structural(&self) -> bool {
    self.kind != XrayChangeKind::Modified
  }
}
