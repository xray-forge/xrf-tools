use serde::{Deserialize, Serialize};

/// Which bloom glows over the frame's bright parts: the engine's, as the look sets it, or the enhanced one.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderBloomMode {
  /// The engine's own (`phase_bloom`), drawn as the look's bloom settings say.
  #[default]
  Engine,
  /// The enhanced bloom in its place: the finished frame's bright parts and self-lit surfaces blurred over six halving
  /// sizes and back, tonemapped and screened over the frame.
  Enhanced,
}
