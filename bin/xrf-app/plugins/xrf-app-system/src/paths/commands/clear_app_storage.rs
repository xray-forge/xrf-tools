use std::io;
use std::path::PathBuf;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;

use crate::paths::AppStorageKind;

/// Removes what the application keeps of a kind that may be rebuilt, the renderer's cache alone; a kind that may not is
/// refused.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "clear_app_storage")]
pub async fn system_clear_app_storage(
  directories: State<'_, AppDirectories>,
  execution: State<'_, ExecutionState>,
  kind: AppStorageKind,
) -> TauriResult<()> {
  let folder: PathBuf = kind
    .get_clearable_folder()
    .ok_or_else(|| format!("{kind:?} cannot be cleared"))?;
  let path: PathBuf = directories
    .get_app_data_directories()
    .into_iter()
    .next()
    .ok_or_else(|| String::from("The application has no data directory"))?
    .join(folder);

  execution
    .run_blocking(
      "Clearing the application's storage",
      move || match std::fs::remove_dir_all(&path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => {
          Err(format!("Could not clear {}: {error}", path.display()))
        }
        _ => {
          log::info!("Cleared {}", path.display());

          Ok(())
        }
      },
    )
    .await?
}
