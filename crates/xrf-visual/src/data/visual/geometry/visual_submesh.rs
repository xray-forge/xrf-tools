use serde::Serialize;
use xrf_ogf::OgfModelType;

use crate::data::visual::geometry::visual_geometry::VisualGeometry;
use crate::data::visual::geometry::visual_skip_cause::VisualSkipCause;
use crate::data::visual::geometry::visual_submesh_content::VisualSubmeshContent;

/// One drawable piece of a visual: a child of a skeleton, or a whole single level visual.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualSubmesh {
  pub index: u32,
  pub model_type: u8,
  pub model_type_label: String,
  /// X-Ray logical texture path, without an extension. A skeleton keeps these on its children rather
  /// than at the top level, which is why a skeleton's own texture chunk is usually absent.
  pub texture_name: Option<String>,
  pub shader_name: Option<String>,
  pub content: VisualSubmeshContent,
}

impl VisualSubmesh {
  /// Whether the submesh stores its geometry as a progressive mesh, which its model type decides.
  pub fn is_progressive(&self) -> bool {
    OgfModelType::from_raw(self.model_type).is_some_and(OgfModelType::is_progressive)
  }

  pub fn geometry(&self) -> Option<&VisualGeometry> {
    match &self.content {
      VisualSubmeshContent::Packed { geometry } => Some(geometry),
      VisualSubmeshContent::Skipped { .. } => None,
    }
  }

  pub fn skipped(&self) -> Option<(VisualSkipCause, &str)> {
    match &self.content {
      VisualSubmeshContent::Packed { .. } => None,
      VisualSubmeshContent::Skipped { cause, reason } => Some((*cause, reason)),
    }
  }

  #[cfg(test)]
  pub(crate) fn skipped_reason(&self) -> Option<&str> {
    self.skipped().map(|(_, reason)| reason)
  }
}
