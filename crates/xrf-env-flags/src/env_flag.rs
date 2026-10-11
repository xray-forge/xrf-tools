use std::path::PathBuf;

use crate::env_flag_kind::EnvFlagKind;
use crate::env_flag_reading::EnvFlagReading;
use crate::env_flag_value::EnvFlagValue;

/// One environment variable the workspace reads or honours: described for its documentation, and read through it.
#[derive(Clone, Copy, Debug)]
pub struct EnvFlag {
  pub name: &'static str,
  pub kind: EnvFlagKind,
  pub value: EnvFlagValue,
  /// What applies while it is unset or set to what it does not take.
  pub default: &'static str,
  /// The binaries or build steps that read it.
  pub readers: &'static [&'static str],
  /// The builds or platforms it is read in, where not all.
  pub condition: Option<&'static str>,
  /// What it does, one sentence.
  pub description: &'static str,
}

impl EnvFlag {
  /// What it is set to in this process, none where unset.
  pub fn get_raw(&self) -> Option<String> {
    std::env::var_os(self.name).map(|value| value.to_string_lossy().into_owned())
  }

  /// A switch's state, none where unset or set to what it does not take, which is warned of.
  pub fn read_switch(&self) -> Option<bool> {
    self.read_as(EnvFlagValue::Switch).map(|value| value == "on")
  }

  /// A choice's name, in the table's spelling; none where unset or set to what it does not take, which is warned of.
  pub fn read_choice(&self) -> Option<&'static str> {
    let EnvFlagValue::Choice(names) = self.value else {
      panic!("{} is not a choice", self.name);
    };
    let chosen: String = self.read_as(self.value)?;

    names.iter().copied().find(|name| *name == chosen)
  }

  /// A path, none where unset or empty.
  pub fn read_path(&self) -> Option<PathBuf> {
    self.read_as(EnvFlagValue::Path).map(PathBuf::from)
  }

  /// Text as set, trimmed; none where unset or empty.
  pub fn read_text(&self) -> Option<String> {
    self.read_as(EnvFlagValue::Text)
  }

  /// How a running binary finds it: set or not, and what it took.
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

  /// The value set, as `expected` takes it, warning where it is set to what it does not take.
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
        "{}={raw:?} is not taken; default: {}. It takes {}",
        self.name,
        self.default,
        self.value.describe()
      );
    }

    interpreted
  }
}
