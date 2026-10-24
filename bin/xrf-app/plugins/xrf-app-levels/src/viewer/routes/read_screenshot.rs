use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use tauri::{AppHandle, Manager};
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::transport::TransportAnswer;

use crate::viewer::LevelViewerFolder;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LevelsReadScreenshotRequest {
  path: PathBuf,
}

/// A screenshot of the screenshots folder as it is stored, for the page to show.
pub(crate) async fn levels_read_screenshot(
  app: AppHandle,
  request: LevelsReadScreenshotRequest,
) -> TauriResult<TransportAnswer> {
  let LevelsReadScreenshotRequest { path } = request;

  LevelViewerFolder::require_screenshot(&app.state::<AppDirectories>(), &path)?;

  let png: Vec<u8> = app
    .state::<ExecutionState>()
    .run_blocking("Reading a screenshot", move || {
      fs::read(&path).map_err(|error| format!("Failed to read the screenshot '{}': {error}", path.display()))
    })
    .await??;

  Ok(TransportAnswer::png(png))
}
