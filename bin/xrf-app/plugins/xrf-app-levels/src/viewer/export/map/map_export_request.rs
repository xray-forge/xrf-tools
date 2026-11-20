use std::path::PathBuf;

use serde::Deserialize;

use crate::viewer::export::map::{MapExportAnnotations, MapExportFormat};

/// A request to re-export a level map capture in another format or size.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MapExportRequest {
  /// The capture's PNG in the screenshots folder, with coverage in alpha.
  pub path: PathBuf,
  pub target: PathBuf,
  pub format: MapExportFormat,
  /// The longer side in pixels, at most the capture's; the shorter keeps the capture's aspect.
  pub long_side: u32,
  /// A colour to flatten the map onto, making it opaque; none keeps the coverage alpha.
  pub backdrop: Option<[u8; 3]>,
  /// How the annotations are included; none for the map alone.
  pub annotations: Option<MapExportAnnotations>,
}
