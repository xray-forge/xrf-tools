use std::ops::Deref;

use crate::env_flag::EnvFlag;
use crate::env_flag_value::EnvFlagValue;

/// A variable read as a switch, on or off.
#[derive(Clone, Copy, Debug)]
pub struct SwitchFlag(EnvFlag);

impl SwitchFlag {
  /// Declares a switch: one whose value is not a switch fails to compile where it is declared.
  pub const fn new(flag: EnvFlag) -> Self {
    assert!(
      matches!(flag.value, EnvFlagValue::Switch),
      "a switch flag takes a switch's value"
    );

    Self(flag)
  }

  /// Its state, or none if it is unset or invalid. An invalid value is logged as a warning.
  pub fn read_switch(&self) -> Option<bool> {
    self.0.read_as(EnvFlagValue::parse_switch)
  }

  pub const fn get_flag(&self) -> &EnvFlag {
    &self.0
  }
}

impl Deref for SwitchFlag {
  type Target = EnvFlag;

  fn deref(&self) -> &EnvFlag {
    &self.0
  }
}
