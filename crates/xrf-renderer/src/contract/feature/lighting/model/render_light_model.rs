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
  /// Linear light: colours authored as display values decoded before they are lit (textures and the irradiance cubes by
  /// the sRGB display's 2.2 power, the weather's and the lamps' colours by their hue alone, keeping the strength the
  /// game balanced them at), the scene held as linear light, and the frame shown through a display transform.
  Physical,
}
