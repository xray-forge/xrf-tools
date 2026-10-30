use serde::{Deserialize, Serialize};

/// Which shadowed lamps draw shafts: those the view's engine gives them, as the game does, or every one, as Monolith
/// forces them on, so a level whose data asks for none can be seen with them.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderLightShafts {
  /// The engine's choice: under vanilla the lamps whose spawn or zone asks, under the extended engine every described
  /// one.
  #[default]
  Engine,
  /// Every shadowed lamp: one that asks at its own shaft values, any other at the engine's defaults.
  Shadowed,
}
