use serde::{Deserialize, Serialize};

/// How the falling rain is drawn.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderRainDropsMode {
  /// The engine's streaks: its rain texture tinted by the weather's rain colour.
  #[default]
  Engine,
  /// Thin drops falling slower where the camera looks, each bending a blurred copy of the scene behind it and lit by a
  /// glint of the light about it and the sky's brightness, and splashes bending what they land on.
  Enhanced,
}
