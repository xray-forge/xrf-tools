use serde::{Deserialize, Serialize};
use xrf_error::XrfResult;

use crate::pack::config::{ArchivePackConfig, ArchivePackDirectory, ArchivePackHeaderEntry};

/// A packing configuration as JSON carries it: the file-owned fields, and nothing else.
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
  /// # Errors
  ///
  /// Returns a parse error for malformed JSON, an unknown key, or a value of the wrong shape.
  pub fn parse(source: &[u8]) -> XrfResult<Self> {
    Ok(serde_json::from_slice(source)?)
  }

  /// Render it as JSON text.
  ///
  /// # Errors
  ///
  /// Returns a serialization error, which the shapes above cannot actually produce.
  pub fn render(&self) -> XrfResult<String> {
    let mut rendered: String = serde_json::to_string_pretty(self)?;

    rendered.push('\n');

    Ok(rendered)
  }

  fn some_if_populated<T: Clone>(values: &[T]) -> Option<Vec<T>> {
    (!values.is_empty()).then(|| values.to_vec())
  }
}

impl From<&ArchivePackConfig> for ArchivePackConfigJson {
  /// Take the file-owned fields of a configuration, leaving the per-run ones behind.
  fn from(config: &ArchivePackConfig) -> Self {
    Self {
      exclude_extensions: Self::some_if_populated(&config.exclude_extensions),
      include_files: Self::some_if_populated(&config.include_files),
      include_directories: Self::some_if_populated(&config.include_directories),
      exclude_directories: Self::some_if_populated(&config.exclude_directories),
      header: config.header.as_deref().map(ArchivePackHeaderEntry::split),
    }
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
      self.header = Some(ArchivePackHeaderEntry::join(entries));
    }

    self
  }
}
