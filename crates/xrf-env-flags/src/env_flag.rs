use std::path::PathBuf;

use crate::env_flag_kind::EnvFlagKind;
use crate::env_flag_reading::EnvFlagReading;
use crate::env_flag_value::EnvFlagValue;

/// One environment variable: its description for the document, and the methods that read it.
#[derive(Clone, Copy, Debug)]
pub struct EnvFlag {
  pub name: &'static str,
  pub kind: EnvFlagKind,
  pub value: EnvFlagValue,
  /// What applies while it is unset or set to an invalid value.
  pub default: &'static str,
  /// The binaries or build steps that read it.
  pub readers: &'static [&'static str],
  /// The builds or platforms that read it, where not all do.
  pub condition: Option<&'static str>,
  /// What it does, in one sentence.
  pub description: &'static str,
}

impl EnvFlag {
  /// Its value in this process, or none if it is unset.
  pub fn get_raw(&self) -> Option<String> {
    std::env::var_os(self.name).map(|value| value.to_string_lossy().into_owned())
  }

  /// A switch's state, or none if it is unset or invalid. An invalid value is logged as a warning.
  pub fn read_switch(&self) -> Option<bool> {
    self.read_as(EnvFlagValue::Switch).map(|value| value == "on")
  }

  /// A choice's name as the table spells it, or none if it is unset or invalid. An invalid value is logged as a
  /// warning.
  pub fn read_choice(&self) -> Option<&'static str> {
    let EnvFlagValue::Choice(names) = self.value else {
      panic!("{} is not a choice", self.name);
    };
    let chosen: String = self.read_as(self.value)?;

    names.iter().copied().find(|name| *name == chosen)
  }

  /// A path, or none if it is unset or empty.
  pub fn read_path(&self) -> Option<PathBuf> {
    self.read_as(EnvFlagValue::Path).map(PathBuf::from)
  }

  /// The text as set, trimmed, or none if it is unset or empty.
  pub fn read_text(&self) -> Option<String> {
    self.read_as(EnvFlagValue::Text)
  }

  /// Its state in this process: whether it is set, and the value that applies.
  pub fn to_reading(&self) -> EnvFlagReading {
    let value: Option<String> = self.get_raw();

    EnvFlagReading {
      name: self.name,
      kind: self.kind,
      description: self.description,
      default: self.default,
      applied: value.as_deref().and_then(|raw| self.value.interpret(raw)),
      value,
    }
  }

  /// The value read as `expected`, logging a warning if it is invalid.
  fn read_as(&self, expected: EnvFlagValue) -> Option<String> {
    assert!(
      std::mem::discriminant(&self.value) == std::mem::discriminant(&expected),
      "{} is read as {:?}, not {expected:?}",
      self.name,
      self.value
    );

    let raw: String = self.get_raw()?;
    let interpreted: Option<String> = self.value.interpret(&raw);

    if interpreted.is_none() && !raw.trim().is_empty() {
      log::warn!(
        "{}={raw:?} is not a valid value, so the default applies ({}). Valid values: {}",
        self.name,
        self.default,
        self.value.describe()
      );
    }

    interpreted
  }
}
