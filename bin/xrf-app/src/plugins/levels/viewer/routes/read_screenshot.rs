use std::fs;
use std::path::PathBuf;

use serde::Deserialize;
use tauri::{AppHandle, Manager};

use crate::core::execution::ExecutionState;
use crate::core::transport::TransportAnswer;
use crate::core::types::TauriResult;
use crate::plugins::levels::viewer::LevelViewerFolder;
use crate::plugins::system::paths::SystemPathsState;

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
  let local_data: Option<PathBuf> = app.state::<SystemPathsState>().local_data.clone();

  LevelViewerFolder::require_screenshot(local_data, &path)?;

  let png: Vec<u8> = app
    .state::<ExecutionState>()
    .run_blocking("Reading a screenshot", move || {
      fs::read(&path).map_err(|error| format!("Failed to read the screenshot '{}': {error}", path.display()))
    })
    .await??;

  Ok(TransportAnswer::png(png))
}
