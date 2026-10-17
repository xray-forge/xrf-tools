use serde::Serialize;
use xrf_archive::{ArchiveFileDescriptor, ArchiveProject};

use crate::commands::archive::list::archive_entry_report::ArchiveEntryReport;
use crate::commands::archive::list::archive_shared_payload_index::ArchiveSharedPayloadIndex;

/// What `archive list` listed.
///
/// Every entry is reported whatever the verbosity: a machine consumer has no `--verbose` to raise,
/// so detail a human would have asked for belongs here unconditionally.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveListReport {
  entries: Vec<ArchiveEntryReport>,
  total: usize,
}

impl ArchiveListReport {
  pub fn new(project: &ArchiveProject, entries: &[&ArchiveFileDescriptor], shared: &ArchiveSharedPayloadIndex) -> Self {
    Self {
      total: entries.len(),
      entries: ArchiveEntryReport::list(project, entries, shared),
    }
  }
}
