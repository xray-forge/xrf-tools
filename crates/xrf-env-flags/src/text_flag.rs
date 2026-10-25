use std::ops::Deref;

use crate::env_flag::EnvFlag;
use crate::env_flag_value::EnvFlagValue;

/// A variable read as free text.
#[derive(Clone, Copy, Debug)]
pub struct TextFlag(EnvFlag);

impl TextFlag {
  /// Declares a text: one whose value is not text fails to compile where it is declared.
  pub const fn new(flag: EnvFlag) -> Self {
    assert!(
      matches!(flag.value, EnvFlagValue::Text),
      "a text flag takes a text's value"
    );

    Self(flag)
  }

  /// The text as set, trimmed, or none if it is unset or empty.
  pub fn read_text(&self) -> Option<String> {
    self.0.read_as(EnvFlagValue::parse_text)
  }

  pub const fn get_flag(&self) -> &EnvFlag {
    &self.0
  }
}

impl Deref for TextFlag {
  type Target = EnvFlag;

  fn deref(&self) -> &EnvFlag {
    &self.0
  }
}
