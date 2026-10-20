use serde::{Deserialize, Serialize};

/// What a view's light is computed in.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderLightModel {
  /// The engine's own: textures, the weather's colours and the lamps lit together as the display-encoded values they
  /// are stored as, as every X-Ray renderer lights them.
  #[default]
  Engine,
  /// Linear light: every colour authored as a display value (textures, the weather's and the lamps' colours, the
  /// irradiance cubes) decoded by the sRGB transfer before it is lit, the scene held as linear light, and the frame
  /// encoded for the display once it is tonemapped.
  Physical,
}
