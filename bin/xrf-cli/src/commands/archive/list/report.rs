use std::collections::HashMap;

use serde::Serialize;
use xrf_archive::{ArchiveFileDescriptor, ArchiveProject, ArchiveSharedPayload};

/// The shared payloads of a project, addressed by the entries located in them.
///
/// Built once per listing so every entry answers in one lookup rather than a scan of the payloads. Lives beside the
/// entry report because the report is what needs it, and `find` borrows both together.
pub(crate) struct ArchiveSharedPayloadIndex<'a> {
  by_name: HashMap<&'a str, &'a ArchiveSharedPayload>,
}

impl<'a> ArchiveSharedPayloadIndex<'a> {
  pub(crate) fn new(payloads: &'a [ArchiveSharedPayload]) -> Self {
    Self {
      by_name: payloads
        .iter()
        .flat_map(|payload| payload.names.iter().map(move |name| (name.as_ref(), payload)))
        .collect(),
    }
  }

  /// The payload `entry` shares with other entries, if it shares one.
  pub(crate) fn get(&self, entry: &ArchiveFileDescriptor) -> Option<&'a ArchiveSharedPayload> {
    self.by_name.get(entry.name.as_ref()).copied()
  }

  /// The other entries located at the same bytes as `entry`, in name order; empty for a payload of its own.
  pub(crate) fn list_others_of(&self, entry: &ArchiveFileDescriptor) -> Vec<String> {
    self
      .get(entry)
      .map(|payload| payload.get_others_of(&entry.name).map(ToString::to_string).collect())
      .unwrap_or_default()
  }

  /// The clause a verbose entry line ends with when the entry shares its payload, and nothing otherwise.
  pub(crate) fn describe_others_of(&self, entry: &ArchiveFileDescriptor) -> String {
    let others: Vec<String> = self.list_others_of(entry);

    if others.is_empty() {
      String::new()
    } else {
      format!(", same payload as {}", others.join(", "))
    }
  }
}

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
  /// Other entries a reader locates at the same stored bytes, by authored name.
  ///
  /// Derived from equal volume, offset, sizes and checksum rather than recorded by whatever packed the set: the format
  /// keeps no alias field, so this says what reads alike and never which entry was written first. Empty for an entry
  /// with a payload of its own.
  shared_with: Vec<String>,
  size_compressed: u64,
  size_real: u64,
  source: String,
}

impl ArchiveEntryReport {
  pub fn new(project: &ArchiveProject, entry: &ArchiveFileDescriptor, shared: &ArchiveSharedPayloadIndex) -> Self {
    Self {
      is_directory: entry.is_directory,
      name: entry.name.to_string(),
      shared_with: shared.list_others_of(entry),
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

  pub fn list(
    project: &ArchiveProject,
    entries: &[&ArchiveFileDescriptor],
    shared: &ArchiveSharedPayloadIndex,
  ) -> Vec<Self> {
    entries.iter().map(|entry| Self::new(project, entry, shared)).collect()
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
  pub fn new(project: &ArchiveProject, entries: &[&ArchiveFileDescriptor], shared: &ArchiveSharedPayloadIndex) -> Self {
    Self {
      total: entries.len(),
      entries: ArchiveEntryReport::list(project, entries, shared),
    }
  }
}
