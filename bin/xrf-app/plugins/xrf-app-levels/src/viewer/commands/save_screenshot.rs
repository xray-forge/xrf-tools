use std::path::PathBuf;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;
use xrf_app_render::CaptureFile;
use xrf_app_render::Renderer;
use xrf_renderer::RenderViewportId;

use crate::viewer::LevelViewerFolder;

/// Write a viewport's next finished frame to a PNG file of the level viewer's screenshots folder: the whole viewport at
/// its drawn size, without its overlays or its selection's outline.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "save_screenshot")]
pub async fn levels_save_screenshot(
  renderer: State<'_, Renderer>,
  directories: State<'_, AppDirectories>,
  execution: State<'_, ExecutionState>,
  viewport: RenderViewportId,
  path: PathBuf,
) -> TauriResult {
  LevelViewerFolder::require_screenshot(&directories, &path)?;

  CaptureFile::save(
    &execution,
    renderer.screenshot_viewport(viewport),
    path,
    format!("Viewport {} drew no frame to take a screenshot of", viewport.0),
  )
  .await
}
