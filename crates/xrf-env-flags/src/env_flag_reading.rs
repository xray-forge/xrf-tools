use serde::Serialize;

use crate::env_flag_kind::EnvFlagKind;

/// A variable as a running binary found it: what it is for and what it was set to, if anything.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvFlagReading {
  pub name: &'static str,
  pub kind: EnvFlagKind,
  /// What it does, one sentence.
  pub description: &'static str,
  /// What applies while it is unset or set to what it does not take.
  pub default: &'static str,
  /// What it was set to, none where unset.
  pub value: Option<String>,
  /// The value it took, in its own spelling: none where unset, or set to what it does not take, so the default applies.
  pub applied: Option<String>,
}

impl EnvFlagReading {
  pub fn is_set(&self) -> bool {
    self.value.is_some()
  }

  /// Whether it is set to what it does not take.
  pub fn is_refused(&self) -> bool {
    self.value.is_some() && self.applied.is_none()
  }
}
