use serde::Serialize;

use crate::data::visual_geometry::VisualGeometry;
use crate::data::visual_skip_cause::VisualSkipCause;

/// Whether a submesh produced drawable geometry, and why not when it did not.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum VisualSubmeshContent {
  Packed { geometry: VisualGeometry },
  Skipped { cause: VisualSkipCause, reason: String },
}
