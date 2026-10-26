use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};

use crate::pack::config::{ArchivePackConfig, ArchivePackHeader};

/// One `[header]` key and its value, in the engine's own spelling.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArchivePackHeaderEntry {
  pub key: String,
  pub value: String,
}

impl ArchivePackHeaderEntry {
  /// One entry written as `<key>=<value>`, split at the first `=` so a value may hold its own, as the engine's own
  /// header does for quoted text.
  ///
  /// # Errors
  ///
  /// Returns an invalid error for an entry with no `=`, or one naming no key: the engine would read the line and find
  /// nothing addressed.
  pub fn parse(entry: &str) -> XrfResult<Self> {
    let (key, value) = entry.split_once('=').ok_or_else(|| {
      XrfError::new_invalid_error(format!(
        "Header entry '{entry}' is not a <key>=<value> pair. Write it as auto_load=true."
      ))
    })?;
    let key: &str = key.trim();

    if key.is_empty() {
      return Err(XrfError::new_invalid_error(format!(
        "Header entry '{entry}' names no key. Write it as auto_load=true."
      )));
    }

    Ok(Self {
      key: key.to_owned(),
      value: value.trim().to_owned(),
    })
  }

  /// Build the verbatim `[header]` text the archive stores as chunk 666.
  pub fn join(entries: &[Self]) -> String {
    let mut header: String = format!("{}\r\n", ArchivePackHeader::SECTION);

    for entry in entries {
      header.push_str(&entry.key);
      header.push_str(" = ");
      header.push_str(&entry.value);
      header.push_str("\r\n");
    }

    header
  }

  /// The default header with `overrides` applied by key, keeping the order the defaults were written in.
  pub fn over_default(overrides: &[Self]) -> Vec<Self> {
    let mut entries: Vec<Self> = Self::split(&ArchivePackHeader::default_text());

    for override_entry in overrides {
      match entries
        .iter_mut()
        .find(|entry| entry.key.eq_ignore_ascii_case(&override_entry.key))
      {
        Some(existing) => existing.value.clone_from(&override_entry.value),
        None => entries.push(override_entry.clone()),
      }
    }

    entries
  }

  /// Split the stored header text back into the pairs it was built from.
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
  pub fn with_header_entries(mut self, entries: &[ArchivePackHeaderEntry]) -> Self {
    self.header = if entries.is_empty() {
      None
    } else {
      Some(ArchivePackHeaderEntry::join(entries))
    };

    self
  }
}
