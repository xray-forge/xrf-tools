use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use xrf_renderer::{RenderCubeCapture, RenderViewportId};

/// A cubemap asked of a viewport, and the cross PNG it is written to.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct CubeCaptureRequest {
  pub viewport: RenderViewportId,
  pub capture: RenderCubeCapture,
  pub path: PathBuf,
}
