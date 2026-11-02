use serde::{Deserialize, Serialize};

/// What colour a surface's albedo is drawn with.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderSurfaceColor {
  /// Its own textures.
  #[default]
  Textured,
  /// One grey for every surface, so the shapes and the light read alone.
  Clay,
  /// A tint of its shader table entry, which tells neighbouring surfaces apart.
  Shader,
}
