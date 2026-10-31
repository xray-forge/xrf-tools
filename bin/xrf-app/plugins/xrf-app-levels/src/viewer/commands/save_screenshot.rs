use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;
use xrf_app_render::CaptureFile;
use xrf_app_render::Renderer;

use crate::viewer::{LevelViewerFolder, ScreenshotSaveRequest};

/// Write a viewport's next finished frame to a PNG file of the level viewer's screenshots folder: the whole viewport at
/// its drawn size, without its overlays or its selection's outline, its level objects' helpers over it where asked.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "save_screenshot")]
pub async fn levels_save_screenshot(
  renderer: State<'_, Renderer>,
  directories: State<'_, AppDirectories>,
  execution: State<'_, ExecutionState>,
  request: ScreenshotSaveRequest,
) -> TauriResult {
  let ScreenshotSaveRequest {
    viewport,
    path,
    screenshot,
  } = request;

  LevelViewerFolder::require_screenshot(&directories, &path)?;

  CaptureFile::save(
    &execution,
    renderer.screenshot_viewport(viewport, screenshot),
    path,
    format!("Viewport {} drew no frame to take a screenshot of", viewport.0),
  )
  .await
}
