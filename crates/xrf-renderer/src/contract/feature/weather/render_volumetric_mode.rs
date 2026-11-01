use serde::{Deserialize, Serialize};

/// How the light the air scatters towards the camera is drawn.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderVolumetricMode {
  /// The engine's own: the sun's shafts marched along each pixel's ray and the shadowed lamps' slices.
  #[default]
  Engine,
  /// A froxel volume: the sun and every local light scattered in one grid along the view, through their shadows, by a
  /// phase function, kept over frames.
  Enhanced,
}
