use crate::key::EnvironmentKey;
use crate::section::EnvironmentSection;
use crate::thunderbolt::ThunderboltCollection;

/// What a list `system.ltx` can add to is keyed by: a section's name and the config it came from.
pub(crate) trait EnvironmentNamed {
  fn get_name(&self) -> &str;
  fn get_file(&self) -> &str;
}

impl<K: EnvironmentKey> EnvironmentNamed for EnvironmentSection<K> {
  fn get_name(&self) -> &str {
    &self.name
  }

  fn get_file(&self) -> &str {
    &self.file
  }
}

impl EnvironmentNamed for ThunderboltCollection {
  fn get_name(&self) -> &str {
    &self.name
  }

  fn get_file(&self) -> &str {
    &self.file
  }
}
