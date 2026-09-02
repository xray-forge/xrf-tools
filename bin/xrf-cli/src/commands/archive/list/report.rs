use serde::Serialize;
use xrf_archive::{ArchiveFileDescriptor, ArchiveProject};

/// One name-table entry, as a machine reads it.
///
/// `name` stays as the name table authored it, `\`-separated: that is an engine identity rather than
/// a host path. `source` is a portable display string, because serializing a `PathBuf` fails outright
/// on a host name that is not valid Unicode and a report describes a run rather than addressing one.
///
/// Lives here rather than beside `find` because listing is the command whose whole answer is entries;
/// `find` reports the same shape, as it already borrows this command's selection rules.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveEntryReport {
  is_directory: bool,
  name: String,
  size_compressed: u64,
  size_real: u64,
  source: String,
}

impl ArchiveEntryReport {
  pub fn new(project: &ArchiveProject, entry: &ArchiveFileDescriptor) -> Self {
    Self {
      is_directory: entry.is_directory,
      name: entry.name.to_string(),
      size_compressed: u64::from(entry.size_compressed),
      size_real: u64::from(entry.size_real),
      // An entry names its volume by position, so the set it belongs to is what turns that back into a path a reader
      // of the report can act on. An entry whose volume the project does not hold reports no source rather than
      // failing a listing over a diagnostic field.
      source: project
        .get_volume_of(entry)
        .map(|volume| xrf_utils::to_portable_path_string(&volume.path))
        .unwrap_or_default(),
    }
  }

  pub fn list(project: &ArchiveProject, entries: &[&ArchiveFileDescriptor]) -> Vec<Self> {
    entries.iter().map(|entry| Self::new(project, entry)).collect()
  }
}

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
  pub fn new(project: &ArchiveProject, entries: &[&ArchiveFileDescriptor]) -> Self {
    Self {
      total: entries.len(),
      entries: ArchiveEntryReport::list(project, entries),
    }
  }
}
