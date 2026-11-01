use serde::{Deserialize, Serialize};

/// A level map exported: its pixels across and down, the capture's own for its annotations alone as an SVG.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct MapExportOutcome {
  pub width: u32,
  pub height: u32,
}
