use crate::{XrayChange, XrayRevision};

/// What one commit or apply changed, and the project revision it made: the next snapshot's.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct XrayChangeSet {
  revision: XrayRevision,
  changes: Vec<XrayChange>,
}

impl XrayChangeSet {
  pub(crate) fn new(revision: XrayRevision, changes: Vec<XrayChange>) -> Self {
    Self { revision, changes }
  }

  /// The project revision of the snapshot these changes made.
  pub fn get_revision(&self) -> XrayRevision {
    self.revision
  }

  /// Every path the changes moved, in the order they were made.
  pub fn get_changes(&self) -> &[XrayChange] {
    &self.changes
  }

  pub fn is_empty(&self) -> bool {
    self.changes.is_empty()
  }
}
