use serde::{Deserialize, Serialize};

/// Whether the sun's light is shadowed by what the frame's depth shows standing between a surface and the sun.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderContactShadowMode {
  /// The engine's own: the sun's cascades alone.
  #[default]
  Engine,
  /// Contact shadows: each pixel's ray towards the sun marched over the frame's depth, under the cascades.
  Enhanced,
}
