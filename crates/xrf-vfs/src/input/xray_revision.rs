use std::collections::HashMap;

use crate::XrayMountId;

/// When an input last changed, as the project revision of the change: zero for what is as it was mounted, which every
/// archived entry always is.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct XrayRevision(u64);

impl XrayRevision {
  /// The revision of everything as it was mounted.
  pub const MOUNTED: Self = Self(0);

  pub fn get(self) -> u64 {
    self.0
  }
}

/// The revisions a world holds for its loose files and indexed directories, stored only where one moved past
/// [`XrayRevision::MOUNTED`].
#[derive(Clone, Debug, Default)]
pub(crate) struct XrayRevisions {
  files: HashMap<XrayMountId, HashMap<String, XrayRevision>>,
  directories: HashMap<XrayMountId, HashMap<String, XrayRevision>>,
}

// todo: bump a file's revision where a commit overwrites it, and its directory's and each ancestor's where a commit
// creates or deletes it, once a world applies a committed change set.
impl XrayRevisions {
  /// When a mount's copy of a normalized logical path last changed.
  pub(crate) fn get_file(&self, mount: XrayMountId, logical_path: &str) -> XrayRevision {
    Self::get(&self.files, mount, logical_path)
  }

  /// When anything under a mount's normalized logical directory, empty for the root, was last created or deleted.
  pub(crate) fn get_directory(&self, mount: XrayMountId, directory: &str) -> XrayRevision {
    Self::get(&self.directories, mount, directory)
  }

  fn get(
    revisions: &HashMap<XrayMountId, HashMap<String, XrayRevision>>,
    mount: XrayMountId,
    path: &str,
  ) -> XrayRevision {
    revisions
      .get(&mount)
      .and_then(|paths| paths.get(path))
      .copied()
      .unwrap_or_default()
  }
}
