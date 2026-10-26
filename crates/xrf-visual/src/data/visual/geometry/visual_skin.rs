use serde::Serialize;

use crate::data::visual::geometry::visual_section::VisualSection;

/// Where one submesh's skinning links sit in the geometry buffer: [`VisualSkin::LINKS`] bone indices and as many
/// weights a vertex, a vertex of fewer links padded with bone nought at no weight.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualSkin {
  pub indices: VisualSection,
  pub weights: VisualSection,
}

impl VisualSkin {
  /// Links a vertex hangs from, which is both the format ceiling (`vertBoned4W`) and the width of a renderer's `vec4`
  /// skin attributes.
  pub const LINKS: usize = 4;
}
