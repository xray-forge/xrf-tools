use serde::Serialize;
use xrf_archive::ArchiveFileDescriptor;

use crate::commands::archive::list::report::ArchiveEntryReport;

/// What `archive find` matched.
///
/// Matches carry the same entry shape a listing reports, since this command narrows a listing rather
/// than answering a different question. The query is echoed back so a stored result says what
/// produced it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveFindReport {
  entries: Vec<ArchiveEntryReport>,
  query: String,
  total: usize,
}

impl ArchiveFindReport {
  pub fn new(query: &str, entries: &[&ArchiveFileDescriptor]) -> Self {
    Self {
      total: entries.len(),
      entries: ArchiveEntryReport::list(entries),
      query: String::from(query),
    }
  }
}
