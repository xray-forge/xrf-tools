use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};

/// One `[header]` key and its value, in the engine's own spelling.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArchivePackHeaderEntry {
  pub key: String,
  pub value: String,
}

impl ArchivePackHeaderEntry {
  pub fn new(key: &str, value: &str) -> Self {
    Self {
      key: key.to_owned(),
      value: value.to_owned(),
    }
  }

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

    Ok(Self::new(key, value.trim()))
  }
}
