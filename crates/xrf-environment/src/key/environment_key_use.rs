use serde::Serialize;

use crate::key::environment_default::EnvironmentDefault;

/// Whether one engine reads a key, and what it holds when a section does not write it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(tag = "use", content = "default", rename_all = "camelCase")]
pub enum EnvironmentKeyUse {
  /// Read with `r_*`: the engine refuses the section without it. The default is what a viewer shows in its place.
  Required(EnvironmentDefault),
  /// Read where written, the default otherwise.
  Optional(EnvironmentDefault),
  /// Required only where another key of the section says so, which the section's reader decides.
  Conditional(EnvironmentDefault),
  /// Not read at all by this engine; kept as written.
  Unread,
}

impl EnvironmentKeyUse {
  /// What the engine holds for the key where the section does not write it; none where the engine does not read it.
  pub fn get_default(self) -> Option<EnvironmentDefault> {
    match self {
      Self::Required(default) | Self::Optional(default) | Self::Conditional(default) => Some(default),
      Self::Unread => None,
    }
  }

  /// Whether this engine reads the key at all.
  pub fn is_read(self) -> bool {
    !matches!(self, Self::Unread)
  }
}
