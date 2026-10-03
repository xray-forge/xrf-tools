use serde::{Deserialize, Serialize};

/// A helper drawn over a viewport's frame, unlit, as the raw colours it names.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum RenderOverlay {
  /// Line segments in renderer space: three floats a vertex, two vertices a segment, and three floats of colour a
  /// vertex.
  Lines {
    positions: Vec<f32>,
    colors: Vec<f32>,
    /// Whether what the scene draws in front hides them.
    is_depth_tested: bool,
  },
  /// A disc in the sky where the light comes from, `size` device pixels across, following the camera and the lighting.
  Sun { color: [f32; 3], size: f32 },
}
