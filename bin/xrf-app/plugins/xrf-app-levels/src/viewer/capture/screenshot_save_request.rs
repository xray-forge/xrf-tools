use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use xrf_renderer::{RenderScreenshot, RenderViewportId};

/// A screenshot asked of a viewport, drawn as `screenshot` asks, and the PNG of the level viewer's screenshots folder
/// it is written to.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotSaveRequest {
  pub viewport: RenderViewportId,
  pub path: PathBuf,
  pub screenshot: RenderScreenshot,
}
