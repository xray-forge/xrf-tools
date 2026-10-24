use std::path::PathBuf;

use serde::Deserialize;

use crate::viewer::cube::CubeExportFormat;

/// A cubemap's capture written again in another shape.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CubeExportRequest {
  /// The capture's cross PNG in the screenshots folder.
  pub path: PathBuf,
  /// The DDS written, or the folder six faces are written into.
  pub target: PathBuf,
  pub format: CubeExportFormat,
  /// Pixels a face takes on a side, at most the capture's.
  pub size: u32,
}
