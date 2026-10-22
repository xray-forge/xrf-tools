use std::path::PathBuf;

use serde::Deserialize;

use crate::plugins::levels::viewer::map_export_format::MapExportFormat;

/// How a level map's annotations, the SVG beside its capture, go into an export of it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum MapExportAnnotations {
  /// Drawn over the map.
  Over,
  /// Alone, clear where they draw nothing.
  Alone,
}

/// A level map's capture written again in another shape.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MapExportRequest {
  /// The capture's PNG in the screenshots folder, its coverage in alpha.
  pub path: PathBuf,
  pub target: PathBuf,
  pub format: MapExportFormat,
  /// Pixels the longer side takes, at most the capture's; the shorter follows the capture's shape.
  pub long_side: u32,
  /// The colour the map is laid over, its alpha made opaque, or none to keep its coverage.
  pub backdrop: Option<[u8; 3]>,
  /// Whether and how its annotations go in, or none for the map alone.
  pub annotations: Option<MapExportAnnotations>,
}
