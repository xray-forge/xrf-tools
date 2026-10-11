use serde::Serialize;

use crate::env_flag_kind::EnvFlagKind;

/// A variable's state in a running binary: what it does, and its value if set.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvFlagReading {
  pub name: &'static str,
  pub kind: EnvFlagKind,
  /// What it does, in one sentence.
  pub description: &'static str,
  /// What applies while it is unset or set to an invalid value.
  pub default: &'static str,
  /// Its value as set, or none if it is unset.
  pub value: Option<String>,
  /// The value that applies, in the table's spelling; none if it is unset or invalid, so the default applies.
  pub applied: Option<String>,
}

impl EnvFlagReading {
  pub fn is_set(&self) -> bool {
    self.value.is_some()
  }

  /// Whether it is set to an invalid value.
  pub fn is_refused(&self) -> bool {
    self.value.is_some() && self.applied.is_none()
  }
}
