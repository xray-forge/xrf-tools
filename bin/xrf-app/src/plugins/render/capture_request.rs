use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use xrf_renderer::{RenderCubeCapture, RenderMapCapture, RenderViewportId};

/// A level map asked of a viewport, and the PNG it is written to.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct MapCaptureRequest {
  pub viewport: RenderViewportId,
  pub capture: RenderMapCapture,
  pub path: PathBuf,
}

/// A cubemap asked of a viewport, around its camera, and the PNG its cross is written to.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct CubeCaptureRequest {
  pub viewport: RenderViewportId,
  pub capture: RenderCubeCapture,
  pub path: PathBuf,
}
