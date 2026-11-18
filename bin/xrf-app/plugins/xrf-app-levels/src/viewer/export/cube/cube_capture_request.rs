use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use xrf_renderer::{RenderCubeCapture, RenderViewportId};

/// A cubemap capture asked of a viewport, written as a cross PNG to `path`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub(crate) struct CubeCaptureRequest {
  pub viewport: RenderViewportId,
  pub capture: RenderCubeCapture,
  pub path: PathBuf,
}
