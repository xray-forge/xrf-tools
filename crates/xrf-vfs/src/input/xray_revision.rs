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

  /// The revision the change after this one makes.
  pub fn next(self) -> Self {
    Self(self.0 + 1)
  }

  /// The revision a change numbered `revision` left, for a test arranging one before anything commits.
  #[cfg(test)]
  pub(crate) const fn of(revision: u64) -> Self {
    Self(revision)
  }
}

/// The revisions a snapshot holds for its loose files and indexed directories, stored only where one moved past
/// [`XrayRevision::MOUNTED`].
#[derive(Clone, Debug, Default)]
pub(crate) struct XrayRevisions {
  files: HashMap<XrayMountId, HashMap<String, XrayRevision>>,
  directories: HashMap<XrayMountId, HashMap<String, XrayRevision>>,
}

impl XrayRevisions {
  /// When a mount's copy of a normalized logical path last changed.
  pub(crate) fn get_file(&self, mount: XrayMountId, logical_path: &str) -> XrayRevision {
    Self::get(&self.files, mount, logical_path)
  }

  /// When anything under a mount's normalized logical directory, empty for the root, was last created or deleted.
  pub(crate) fn get_directory(&self, mount: XrayMountId, directory: &str) -> XrayRevision {
    Self::get(&self.directories, mount, directory)
  }

  /// Moves a mount's copy of a normalized logical path to `revision`.
  pub(crate) fn set_file(&mut self, mount: XrayMountId, logical_path: &str, revision: XrayRevision) {
    self
      .files
      .entry(mount)
      .or_default()
      .insert(logical_path.to_owned(), revision);
  }

  /// Moves every directory above a normalized logical path in a mount to `revision`, the logical root included: what
  /// sits under each of them changed.
  pub(crate) fn set_directories_above(&mut self, mount: XrayMountId, logical_path: &str, revision: XrayRevision) {
    let directories: &mut HashMap<String, XrayRevision> = self.directories.entry(mount).or_default();

    directories.insert(String::new(), revision);

    for (index, _) in logical_path.match_indices('\\') {
      directories.insert(logical_path[..index].to_owned(), revision);
    }
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
