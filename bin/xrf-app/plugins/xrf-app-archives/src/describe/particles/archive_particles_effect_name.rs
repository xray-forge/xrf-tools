use std::collections::HashSet;

use serde::Serialize;

/// An effect a group names, and whether this library is where it is defined.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveParticlesEffectName {
  pub name: String,
  pub is_defined: bool,
}

impl ArchiveParticlesEffectName {
  pub(super) fn of(name: &str, effects: &HashSet<&str>) -> Self {
    Self {
      name: name.to_owned(),
      is_defined: effects.contains(name),
    }
  }

  /// The same, for a slot that is free to name nothing - which most of the three child slots do.
  pub(super) fn of_optional(name: &str, effects: &HashSet<&str>) -> Option<Self> {
    (!name.is_empty()).then(|| Self::of(name, effects))
  }
}
