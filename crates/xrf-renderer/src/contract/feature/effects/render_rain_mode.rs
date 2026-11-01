use serde::{Deserialize, Serialize};

/// Whether puddles gather on the ground when it rains, over the engine's own wetting of the surfaces either way.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderRainMode {
  /// The engine's own wetting alone: splashes and running water near the camera, wet only while it rains.
  #[default]
  Engine,
  /// Puddles gathering on flat, low terrain over the engine's own wetting, filling with the rain and draining after,
  /// their water reflecting as the reflections trace.
  Enhanced,
}
