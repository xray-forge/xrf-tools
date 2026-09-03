use crate::archive_shared_payload::ArchiveSharedPayload;
use crate::project::archive_project::ArchiveProject;

impl ArchiveProject {
  /// The payloads more than one entry of this project locates, in volume and offset order.
  ///
  /// Derived on request from the merged name table and kept nowhere, so it can never disagree with the entries. An
  /// entry a later volume shadowed is not in the table and is not counted; see [`ArchiveSharedPayload`] for why this
  /// is a reader's observation rather than a packer's record.
  pub fn list_shared_payloads(&self) -> Vec<ArchiveSharedPayload> {
    ArchiveSharedPayload::derive(self.files.values())
  }
}
