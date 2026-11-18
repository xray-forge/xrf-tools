use std::path::PathBuf;

use serde::Deserialize;

use crate::viewer::export::cube::CubeExportFormat;

/// A request to re-export a cubemap capture in another format or size.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CubeExportRequest {
  /// The capture's cross PNG in the screenshots folder.
  pub path: PathBuf,
  /// The DDS file, or the folder the six faces go into.
  pub target: PathBuf,
  pub format: CubeExportFormat,
  /// Face side in pixels, at most the capture's.
  pub size: u32,
}
