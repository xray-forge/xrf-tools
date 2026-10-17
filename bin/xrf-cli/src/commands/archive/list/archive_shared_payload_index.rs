use std::collections::HashMap;

use xrf_archive::{ArchiveFileDescriptor, ArchiveSharedPayload};

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
