use crate::env_flag_kind::EnvFlagKind;
use crate::env_flag_reading::EnvFlagReading;
use crate::env_flag_value::EnvFlagValue;

/// One environment variable as the document describes it; it is read through the typed flag declaring it.
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

  /// The value as `parse` reads it, logging a warning if it is set but invalid.
  pub(crate) fn read_as<T>(&self, parse: impl FnOnce(&str) -> Option<T>) -> Option<T> {
    let raw: String = self.get_raw()?;
    let parsed: Option<T> = parse(&raw);

    if parsed.is_none() && !raw.trim().is_empty() {
      log::warn!(
        "{}={raw:?} is not a valid value, so the default applies ({}). Valid values: {}",
        self.name,
        self.default,
        self.value.describe()
      );
    }

    parsed
  }
}
