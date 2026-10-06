use serde::{Deserialize, Serialize};

/// What one pass of a viewport's frames cost on the GPU.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderPassCost {
  /// The pass, as the frame names it.
  pub name: String,
  /// Mean GPU milliseconds a frame over the report's span, a frame it did not run in counting nought.
  pub gpu_time: f32,
}
