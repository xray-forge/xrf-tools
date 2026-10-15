use std::ops::Deref;

use crate::env_flag::EnvFlag;
use crate::env_flag_value::EnvFlagValue;

/// A variable read as one of a set of names.
#[derive(Clone, Copy, Debug)]
pub struct ChoiceFlag {
  flag: EnvFlag,
  names: &'static [&'static str],
}

impl ChoiceFlag {
  /// Declares a choice: one whose value is not a choice fails to compile where it is declared.
  pub const fn new(flag: EnvFlag) -> Self {
    let EnvFlagValue::Choice(names) = flag.value else {
      panic!("a choice flag takes a choice's value");
    };

    Self { flag, names }
  }

  /// The name chosen as the table spells it, or none if it is unset or invalid. An invalid value is logged as a
  /// warning.
  pub fn read_choice(&self) -> Option<&'static str> {
    let chosen: String = self.flag.read_interpreted()?;

    self.names.iter().copied().find(|name| *name == chosen)
  }

  pub const fn get_flag(&self) -> &EnvFlag {
    &self.flag
  }
}

impl Deref for ChoiceFlag {
  type Target = EnvFlag;

  fn deref(&self) -> &EnvFlag {
    &self.flag
  }
}
