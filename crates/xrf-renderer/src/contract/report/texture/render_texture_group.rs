use serde::{Deserialize, Serialize};

/// What of a viewport's scene samples a texture.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderTextureGroup {
  /// The static surfaces: the level's geometry and what stands on it.
  #[default]
  Surfaces,
  /// The grass's tufts, and none of the surfaces.
  Grass,
}
