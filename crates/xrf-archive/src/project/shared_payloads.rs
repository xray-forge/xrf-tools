use crate::archive_shared_payload::ArchiveSharedPayload;
use crate::project::archive_project::ArchiveProject;

impl ArchiveProject {
  /// The payloads more than one entry of this project locates, in volume and offset order.
  pub fn list_shared_payloads(&self) -> Vec<ArchiveSharedPayload> {
    ArchiveSharedPayload::derive(self.files.values())
  }
}
