use std::path::PathBuf;

use tauri::State;
use xrf_renderer::RenderViewportId;

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::levels::viewer::LevelViewerFolder;
use crate::plugins::render::capture_file::save_capture_file;
use crate::plugins::render::state::RenderState;
use crate::plugins::system::paths::SystemPathsState;

/// Write a viewport's next finished frame to a PNG file of the level viewer's screenshots folder: the whole viewport at
/// its drawn size, without its overlays or its selection's outline.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "save_screenshot"))]
#[tauri::command(rename = "save_screenshot")]
pub async fn render_save_screenshot(
  state: State<'_, RenderState>,
  paths: State<'_, SystemPathsState>,
  execution: State<'_, ExecutionState>,
  viewport: RenderViewportId,
  path: PathBuf,
) -> TauriResult {
  LevelViewerFolder::require_screenshot(paths.local_data.clone(), &path)?;

  save_capture_file(
    &execution,
    state.renderer.screenshot_viewport(viewport),
    path,
    format!("Viewport {} drew no frame to take a screenshot of", viewport.0),
  )
  .await
}
