use std::collections::BTreeMap;

use xrf_error::{XrfError, XrfResult};
use xrf_game_config::{ConfigFile, ConfigSource};

/// Configs held as text: a `system.ltx` and files by path.
pub(crate) struct TextSource {
  system: ConfigFile,
  files: BTreeMap<String, String>,
}

impl TextSource {
  pub(crate) fn new(system: &str, files: &[(&str, &str)]) -> Self {
    Self {
      system: ConfigFile::parse("system.ltx", system).expect("the test system.ltx reads"),
      files: files
        .iter()
        .map(|(path, text)| (String::from(*path), String::from(*text)))
        .collect(),
    }
  }
}

impl ConfigSource for TextSource {
  fn get_system(&self) -> &ConfigFile {
    &self.system
  }

  fn read_file(&self, path: &str) -> XrfResult<ConfigFile> {
    let text: &String = self
      .files
      .get(path)
      .ok_or_else(|| XrfError::new_invalid_error(format!("no {path}")))?;

    ConfigFile::parse(path, text)
  }
}
