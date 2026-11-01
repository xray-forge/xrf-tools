use serde::{Deserialize, Serialize};

/// Where a view's frame is tonemapped.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderTonemapMode {
  /// The engine's own: combine tonemaps the lit scene, and every forward pass after it draws tonemapped colour.
  #[default]
  Engine,
  /// The scene held as linear light at its exposure through every pass and the resolve, the engine's curve applied
  /// once by the window pass at the viewport's size, as an upscaler wants its input.
  Enhanced,
}
