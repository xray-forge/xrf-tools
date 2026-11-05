use serde::{Deserialize, Serialize};

/// The mouse buttons a drag holds, by their `PointerEvent` roles: a left-handed mouse swaps the two sides.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldInputButtons {
  /// The main button, usually the left.
  pub main: bool,
  /// The secondary button, usually the right.
  pub secondary: bool,
  /// The auxiliary button, usually the wheel's press.
  pub auxiliary: bool,
}

impl WorldInputButtons {
  pub const MAIN: Self = Self {
    main: true,
    secondary: false,
    auxiliary: false,
  };

  pub const SECONDARY: Self = Self {
    main: false,
    secondary: true,
    auxiliary: false,
  };

  pub const AUXILIARY: Self = Self {
    main: false,
    secondary: false,
    auxiliary: true,
  };

  pub const MAIN_AND_SECONDARY: Self = Self {
    main: true,
    secondary: true,
    auxiliary: false,
  };
}
