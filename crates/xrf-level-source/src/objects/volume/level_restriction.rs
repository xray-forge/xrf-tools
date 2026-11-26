use serde::Serialize;

/// Whom a logic volume keeps where, `RestrictionSpace::ERestrictorTypes` with the default and the scripted kinds
/// folded together.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelRestriction {
  /// Restricts no one by itself.
  None,
  /// Keeps those it restricts outside it.
  Out,
  /// Keeps those it restricts inside it.
  In,
}

impl From<u8> for LevelRestriction {
  /// `eDefaultRestrictorTypeOut` and `eRestrictorTypeOut` (1, 5) keep out, `eDefaultRestrictorTypeIn` and
  /// `eRestrictorTypeIn` (2, 4) keep in, and every other value restricts nothing.
  fn from(restrictor_type: u8) -> Self {
    match restrictor_type {
      1 | 5 => Self::Out,
      2 | 4 => Self::In,
      _ => Self::None,
    }
  }
}
