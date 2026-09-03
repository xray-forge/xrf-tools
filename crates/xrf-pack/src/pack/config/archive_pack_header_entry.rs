use serde::{Deserialize, Serialize};

use crate::pack::config::ArchivePackConfig;

/// One `[header]` key and its value, in the engine's own spelling.
///
/// The section is copied into the archive verbatim and the engine parses it, so neither half is renamed on the way
/// through a configuration file: `auto_load` stays `auto_load`, whatever the surrounding field names look like.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArchivePackHeaderEntry {
  pub key: String,
  pub value: String,
}

impl ArchivePackHeaderEntry {
  /// Build the verbatim `[header]` text the archive stores as chunk 666.
  ///
  /// Written in the engine's line ending because the engine reads these bytes, not because anything here does.
  pub fn join(entries: &[Self]) -> String {
    let mut header: String = String::from("[header]\r\n");

    for entry in entries {
      header.push_str(&entry.key);
      header.push_str(" = ");
      header.push_str(&entry.value);
      header.push_str("\r\n");
    }

    header
  }

  /// Split the stored header text back into the pairs it was built from.
  ///
  /// The header is kept as text because the archive stores it verbatim; this reads it only well enough to round trip
  /// through a configuration file. A line carrying no `=` names nothing and is dropped.
  pub fn split(header: &str) -> Vec<Self> {
    header
      .lines()
      .filter_map(|line| line.split_once('='))
      .map(|(key, value)| Self {
        key: key.trim().to_string(),
        value: value.trim().to_string(),
      })
      .collect()
  }
}

impl ArchivePackConfig {
  /// Replace the header with the entries given, or drop it entirely when none are.
  ///
  /// Replaces rather than merges, matching a present `[header]` section: a caller naming its own keys means those and
  /// no others, and silently keeping a default `entry_point` beside them would mount an archive somewhere nobody asked
  /// for.
  pub fn with_header_entries(mut self, entries: &[ArchivePackHeaderEntry]) -> Self {
    self.header = if entries.is_empty() {
      None
    } else {
      Some(ArchivePackHeaderEntry::join(entries))
    };

    self
  }
}
