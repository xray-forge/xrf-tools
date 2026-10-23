use std::any::TypeId;

use crate::XrayRevision;

/// What identifies a retained value: its type, and the copy it was parsed from at the revision it was read at.
///
/// The copy is named by the opened source holding it and its path there, not by the logical path a search asked for:
/// any search answering with that copy is served the same value, and a mount winning the path later is a different copy
/// rather than a reason to drop this one. Named by the mount's opening rather than its id or root path, because the forks
/// of a snapshot share one cache, two of them may mount different sources at one id, and two single volumes of one
/// directory share a root path.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct XrayCacheKey {
  type_id: TypeId,
  opening: u64,
  path: String,
  revision: XrayRevision,
}

impl XrayCacheKey {
  /// Identifies the value of one type parsed from the copy at `path` in the source a mount opened as `opening`, as it
  /// was at `revision`.
  pub(crate) fn of<T: 'static>(opening: u64, path: &str, revision: XrayRevision) -> Self {
    Self {
      type_id: TypeId::of::<T>(),
      opening,
      path: path.to_owned(),
      revision,
    }
  }

  /// Whether this key names a value parsed from one copy, whatever its type or revision.
  pub(crate) fn is_copy(&self, opening: u64, path: &str) -> bool {
    self.opening == opening && self.path == path
  }

  pub(crate) fn get_opening(&self) -> u64 {
    self.opening
  }

  pub(crate) fn get_path(&self) -> &str {
    &self.path
  }

  pub(crate) fn get_revision(&self) -> XrayRevision {
    self.revision
  }
}
