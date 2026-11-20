use serde::{Deserialize, Serialize};

/// An exported level map's size in pixels; the capture's size for an annotations-only SVG.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub(crate) struct MapExportOutcome {
  pub width: u32,
  pub height: u32,
}
