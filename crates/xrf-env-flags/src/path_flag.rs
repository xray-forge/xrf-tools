use std::ffi::OsString;
use std::ops::Deref;
use std::path::PathBuf;

use crate::env_flag::EnvFlag;
use crate::env_flag_value::EnvFlagValue;

/// A variable read as a path.
#[derive(Clone, Copy, Debug)]
pub struct PathFlag(EnvFlag);

impl PathFlag {
  /// Declares a path: one whose value is not a path fails to compile where it is declared.
  pub const fn new(flag: EnvFlag) -> Self {
    assert!(
      matches!(flag.value, EnvFlagValue::Path),
      "a path flag takes a path's value"
    );

    Self(flag)
  }

  /// The path, trimmed, or none if it is unset or empty; one that is not Unicode as it is set, which a text reading of it
  /// would change.
  pub fn read_path(&self) -> Option<PathBuf> {
    let raw: OsString = std::env::var_os(self.0.name)?;

    match raw.to_str() {
      Some(text) => (!text.trim().is_empty()).then(|| PathBuf::from(text.trim())),
      None => Some(PathBuf::from(raw)),
    }
  }

  pub const fn get_flag(&self) -> &EnvFlag {
    &self.0
  }
}

impl Deref for PathFlag {
  type Target = EnvFlag;

  fn deref(&self) -> &EnvFlag {
    &self.0
  }
}
