use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::Ltx;
use xrf_utils::{format_path, write_file_staged};

use crate::pack::config::ArchivePackHeaderEntry;
use crate::patch::config::{ArchivePatchConfig, ArchivePatchConfigFormat};

/// A patching configuration as a file carries it: the file-owned fields, and nothing else.
///
/// [`ArchivePatchConfig`] is the whole run, and most of it — what is compared, where it is published, under what name,
/// which shape it is read in — belongs to the invocation or the open form rather than to a file shared between
/// machines. What is left is the comparison scope and the header, the same split `ArchivePackConfigJson` makes.
///
/// Every field is optional, and an absent one means "leave what the caller already holds", so importing is a layering
/// step rather than a replacement and an explicit option keeps winning over it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArchivePatchConfigJson {
  /// Logical prefixes the comparison is restricted to.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub include: Option<Vec<String>>,
  /// Logical prefixes dropped from the comparison.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub ignore: Option<Vec<String>>,
  /// Extension patterns that keep a file out of the comparison, matched with the dot.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub exclude_extensions: Option<Vec<String>>,
  /// `[header]` entries written into the published volumes.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub header: Option<Vec<ArchivePackHeaderEntry>>,
}

impl ArchivePatchConfigJson {
  /// Read one out of its JSON text.
  ///
  /// Strict on purpose: an unknown key is refused rather than ignored, because a file whose `ignore` was typed
  /// `ignored` would otherwise compare something other than what it describes, and say nothing.
  ///
  /// # Errors
  ///
  /// Returns a parse error for malformed JSON, an unknown key, or a value of the wrong shape.
  pub fn parse(source: &[u8]) -> XrfResult<Self> {
    Ok(serde_json::from_slice(source)?)
  }

  /// Render it as JSON text.
  ///
  /// Deterministic: declaration order, absent fields omitted rather than written as null, and a trailing newline, so
  /// a checked-in configuration only changes when its contents do.
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
  /// An empty collection is written as absent rather than as an empty list, so a round trip leaves a caller's own
  /// values alone rather than clearing them.
  pub fn from_config(config: &ArchivePatchConfig) -> Self {
    Self {
      include: Self::some_if_populated(config.include.clone()),
      ignore: Self::some_if_populated(config.ignore.clone()),
      exclude_extensions: Self::some_if_populated(config.exclude_extensions.clone()),
      header: config.header.as_deref().map(ArchivePackHeaderEntry::split),
    }
  }

  fn some_if_populated<T>(values: Vec<T>) -> Option<Vec<T>> {
    (!values.is_empty()).then_some(values)
  }
}

impl ArchivePatchConfig {
  /// Apply the file-owned fields of a parsed configuration over the one the caller holds.
  #[must_use]
  pub fn with_json(mut self, json: &ArchivePatchConfigJson) -> Self {
    if let Some(prefixes) = &json.include {
      self.include = prefixes.clone();
    }

    if let Some(prefixes) = &json.ignore {
      self.ignore = prefixes.clone();
    }

    if let Some(extensions) = &json.exclude_extensions {
      self.exclude_extensions = extensions.clone();
    }

    if let Some(entries) = &json.header {
      self.header = Some(ArchivePackHeaderEntry::join(entries));
    }

    self
  }

  /// Take the file-owned fields of this configuration in the shape a file carries.
  pub fn to_json(&self) -> ArchivePatchConfigJson {
    ArchivePatchConfigJson::from_config(self)
  }

  /// Apply a patching configuration file over this configuration, in whichever format its extension names.
  ///
  /// # Errors
  ///
  /// Returns an invalid error for a path whose extension names no supported format, and a read or parse error for a
  /// file that cannot be read as the format it claims.
  pub fn with_config_file<P: AsRef<Path>>(self, path: P) -> XrfResult<Self> {
    let path: &Path = path.as_ref();

    match ArchivePatchConfigFormat::from_path(path)? {
      ArchivePatchConfigFormat::Ltx => self.with_ltx(&Ltx::read_from_file_standard(path)?),
      ArchivePatchConfigFormat::Json => {
        let json: ArchivePatchConfigJson = ArchivePatchConfigJson::parse(&fs::read(path)?).map_err(|error| {
          XrfError::new_parsing_error(format!(
            "Failed to parse patching configuration '{}': {error}",
            format_path(path)
          ))
        })?;

        Ok(self.with_json(&json))
      }
    }
  }

  /// Write the file-owned fields out as a patching configuration file, in whichever format the path names.
  ///
  /// # Errors
  ///
  /// Returns an invalid error for an unsupported extension, and an IO error when the file cannot be published.
  pub fn write_config_to_path<P: AsRef<Path>>(&self, path: P) -> XrfResult {
    let path: &Path = path.as_ref();

    let rendered: Vec<u8> = match ArchivePatchConfigFormat::from_path(path)? {
      ArchivePatchConfigFormat::Ltx => self.to_ltx_bytes()?,
      ArchivePatchConfigFormat::Json => self.to_json().render()?.into_bytes(),
    };

    write_file_staged(path, &rendered).map_err(|error| {
      XrfError::new_io_error(
        format!(
          "Failed to write the patching configuration to '{}': {error}",
          format_path(path)
        ),
        error.kind(),
      )
    })
  }
}
