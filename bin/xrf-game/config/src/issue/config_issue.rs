use std::fmt::{self, Display, Formatter};

/// Something of the configs that did not read as the game reads it: where (the file, the section, the key where one
/// is to blame) and what, as the game logs it and a modder fixes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigIssue {
  /// The config's path under `configs`, as `system.ltx` or `misc\simulation.ltx`, or what the text came from.
  pub file: String,
  /// The section, none for an issue of the whole file.
  pub section: Option<String>,
  /// The key, none for an issue of the whole section.
  pub key: Option<String>,
  pub message: String,
}

impl Display for ConfigIssue {
  /// `file [section] key: message`, each part there is.
  fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
    write!(formatter, "{}", self.file)?;

    if let Some(section) = &self.section {
      write!(formatter, " [{section}]")?;
    }

    if let Some(key) = &self.key {
      write!(formatter, " {key}")?;
    }

    write!(formatter, ": {}", self.message)
  }
}
