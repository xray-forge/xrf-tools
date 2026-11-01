use serde::{Deserialize, Serialize};

/// What fog closes the level in: the weather's distance fog alone, or that fog thickened low down and scattering the
/// frame's light.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderFogMode {
  /// The engine's own: the weather's distance fog.
  #[default]
  Engine,
  /// The distance fog thickened below a height and tinted towards the sun there, with the bright parts of the frame
  /// blurred into where it lies.
  Enhanced,
}
