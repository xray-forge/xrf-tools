use serde::Serialize;

use crate::data::visual_section::VisualSection;

/// Where one submesh's skinning links sit in the geometry buffer.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualSkin {
  pub indices: VisualSection,
  pub weights: VisualSection,
}
