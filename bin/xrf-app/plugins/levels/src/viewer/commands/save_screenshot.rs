use std::path::PathBuf;

use tauri::State;
use xrf_app_core::app_directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::types::TauriResult;
use xrf_app_render::capture_file::save_capture_file;
use xrf_app_render::renderer::Renderer;
use xrf_renderer::RenderViewportId;

use crate::viewer::LevelViewerFolder;

/// Write a viewport's next finished frame to a PNG file of the level viewer's screenshots folder: the whole viewport at
/// its drawn size, without its overlays or its selection's outline.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "save_screenshot"))]
#[tauri::command(rename = "save_screenshot")]
pub async fn levels_save_screenshot(
  renderer: State<'_, Renderer>,
  directories: State<'_, AppDirectories>,
  execution: State<'_, ExecutionState>,
  viewport: RenderViewportId,
  path: PathBuf,
) -> TauriResult {
  LevelViewerFolder::require_screenshot(&directories, &path)?;

  save_capture_file(
    &execution,
    renderer.screenshot_viewport(viewport),
    path,
    format!("Viewport {} drew no frame to take a screenshot of", viewport.0),
  )
  .await
}
