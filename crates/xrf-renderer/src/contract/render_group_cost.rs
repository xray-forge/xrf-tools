use serde::{Deserialize, Serialize};

/// What one of the frame graph's encode groups cost the render thread a frame, mean milliseconds over a report's span:
/// the groups record and are encoded at once on threads of their own, so the longest bounds the frame's encoding.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGroupCost {
  pub name: String,
  /// Recording its passes.
  pub record: f32,
  /// wgpu's encoding of what was recorded, in the encoder's `finish`.
  pub finish: f32,
}
