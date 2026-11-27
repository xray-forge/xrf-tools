use std::collections::BTreeMap;

use xrf_error::{XrfError, XrfResult};
use xrf_ltx::Ltx;

use crate::config::simulation_config_source::SimulationConfigSource;

/// Configs held as text: a `system.ltx` and files by path.
pub(crate) struct ConfigSource {
  system: Ltx,
  files: BTreeMap<String, String>,
}

impl ConfigSource {
  pub(crate) fn new(system: &str, files: &[(&str, &str)]) -> Self {
    Self {
      system: Ltx::read_from_str(system).expect("the test system.ltx reads"),
      files: files
        .iter()
        .map(|(path, text)| (String::from(*path), String::from(*text)))
        .collect(),
    }
  }
}

impl SimulationConfigSource for ConfigSource {
  fn get_system(&self) -> &Ltx {
    &self.system
  }

  fn read_config(&self, path: &str) -> XrfResult<Ltx> {
    let text: &String = self
      .files
      .get(path)
      .ok_or_else(|| XrfError::new_invalid_error(format!("no {path}")))?;

    Ltx::read_from_str(text)
  }
}
