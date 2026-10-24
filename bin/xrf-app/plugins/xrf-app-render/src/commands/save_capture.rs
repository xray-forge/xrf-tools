use std::path::PathBuf;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_renderer::RenderViewportId;

use crate::{CaptureFile, Renderer};

/// Write a viewport's next presented frame to a PNG file, as the renderer drew it rather than as the screen shows it.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "save_capture"))]
#[tauri::command(rename = "save_capture")]
pub async fn render_save_capture(
  renderer: State<'_, Renderer>,
  execution: State<'_, ExecutionState>,
  viewport: RenderViewportId,
  path: PathBuf,
) -> TauriResult {
  CaptureFile::save(
    &execution,
    renderer.capture_viewport(viewport),
    path,
    format!("Viewport {} drew no frame to capture", viewport.0),
  )
  .await
}
