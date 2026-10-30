use std::path::PathBuf;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;

use crate::paths::AppStorageUsage;

/// What the application keeps in its own data directory, the one its caches go to, by kind.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "get_app_storage")]
pub async fn system_get_app_storage(
  directories: State<'_, AppDirectories>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<AppStorageUsage> {
  let root: PathBuf = directories
    .get_app_data_directories()
    .into_iter()
    .next()
    .ok_or_else(|| String::from("The application has no data directory"))?;

  // Every file is measured, which grows with the caches.
  execution
    .run_blocking("Measuring the application's storage", move || {
      AppStorageUsage::measure(&root)
    })
    .await
}
