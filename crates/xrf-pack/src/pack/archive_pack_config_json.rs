use serde::{Deserialize, Serialize};
use xrf_error::XrfResult;

use crate::pack::archive_pack_config::{ArchivePackConfig, ArchivePackDirectory};

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

/// A packing configuration as JSON carries it: the file-owned fields, and nothing else.
///
/// [`ArchivePackConfig`] is the whole run — source, destination, volume name, mode, volume size — and most of it
/// belongs to the invocation or the open form rather than to a file shared between machines. This is the subset an
/// LTX has always carried, given a shape of its own so JSON and LTX are two serializations of one payload rather
/// than two unrelated features.
///
/// Every field is optional, and an absent one means the same thing an absent LTX section does: leave what the caller
/// already holds. That is what makes importing a file a layering step rather than a replacement, so the explicit
/// options of a command line or a form keep winning over it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArchivePackConfigJson {
  /// Extension patterns from `[options] exclude_exts`, matched against the extension with its dot.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub exclude_extensions: Option<Vec<String>>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub include_files: Option<Vec<String>>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub include_directories: Option<Vec<ArchivePackDirectory>>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub exclude_directories: Option<Vec<ArchivePackDirectory>>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub header: Option<Vec<ArchivePackHeaderEntry>>,
}

impl ArchivePackConfigJson {
  /// Read one out of its JSON text.
  ///
  /// Strict on purpose: an unknown key is refused rather than ignored, because a file whose `includeFiles` was typed
  /// `includeFile` would otherwise pack a different archive than the one it describes, and say nothing.
  ///
  /// # Errors
  ///
  /// Returns a parse error for malformed JSON, an unknown key, or a value of the wrong shape.
  pub fn parse(source: &[u8]) -> XrfResult<Self> {
    Ok(serde_json::from_slice(source)?)
  }

  /// Render it as JSON text.
  ///
  /// Deterministic: fields are written in declaration order, an absent one is omitted rather than written as null,
  /// and the text ends with a newline the way every other text artifact here does. Two runs over equal values
  /// produce equal bytes, so a checked-in configuration only changes when its contents do.
  ///
  /// # Errors
  ///
  /// Returns a serialization error, which the shapes above cannot actually produce.
  pub fn render(&self) -> XrfResult<String> {
    let mut rendered: String = serde_json::to_string_pretty(self)?;

    rendered.push('\n');

    Ok(rendered)
  }

  /// Take the file-owned fields of a configuration, leaving the per-run ones behind.
  ///
  /// An empty collection is written as absent rather than as an empty list, matching an LTX export omitting a
  /// section it has no entries for. Both serializations therefore say "nothing to apply here" the same way, and a
  /// round trip through either leaves a caller's own values alone.
  pub fn from_config(config: &ArchivePackConfig) -> Self {
    Self {
      exclude_extensions: Self::some_if_populated(config.exclude_extensions.clone()),
      include_files: Self::some_if_populated(config.include_files.clone()),
      include_directories: Self::some_if_populated(config.include_directories.clone()),
      exclude_directories: Self::some_if_populated(config.exclude_directories.clone()),
      header: config.header.as_deref().map(Self::split_header),
    }
  }

  /// Rebuild the verbatim `[header]` text the archive stores as chunk 666.
  ///
  /// Written in the engine's line ending because the engine reads these bytes, not because anything here does.
  pub(crate) fn join_header(entries: &[ArchivePackHeaderEntry]) -> String {
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
  /// through a configuration file. A line carrying no `=` names nothing and is dropped, which is what the LTX
  /// projection has always done with it.
  pub(crate) fn split_header(header: &str) -> Vec<ArchivePackHeaderEntry> {
    header
      .lines()
      .filter_map(|line| line.split_once('='))
      .map(|(key, value)| ArchivePackHeaderEntry {
        key: key.trim().to_string(),
        value: value.trim().to_string(),
      })
      .collect()
  }

  fn some_if_populated<T>(values: Vec<T>) -> Option<Vec<T>> {
    (!values.is_empty()).then_some(values)
  }
}

impl ArchivePackConfig {
  /// Apply the file-owned fields of a parsed JSON configuration over the one the caller holds.
  ///
  /// The counterpart of [`ArchivePackConfig::with_ltx`], and layered the same way: a field the JSON does not carry
  /// leaves what is already there.
  pub fn with_json(mut self, json: &ArchivePackConfigJson) -> Self {
    if let Some(extensions) = &json.exclude_extensions {
      self.exclude_extensions = extensions.clone();
    }

    if let Some(files) = &json.include_files {
      self.include_files = files.clone();
    }

    if let Some(directories) = &json.include_directories {
      self.include_directories = directories.clone();
    }

    if let Some(directories) = &json.exclude_directories {
      self.exclude_directories = directories.clone();
    }

    if let Some(entries) = &json.header {
      self.header = Some(ArchivePackConfigJson::join_header(entries));
    }

    self
  }

  /// Take the file-owned fields of this configuration in the shape JSON carries.
  pub fn to_json(&self) -> ArchivePackConfigJson {
    ArchivePackConfigJson::from_config(self)
  }
}
