use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use xrf_renderer::{RenderMapCapture, RenderViewportId};

/// A level map capture asked of a viewport, written as a PNG to `path`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub(crate) struct MapCaptureRequest {
  pub viewport: RenderViewportId,
  pub capture: RenderMapCapture,
  pub path: PathBuf,
}
