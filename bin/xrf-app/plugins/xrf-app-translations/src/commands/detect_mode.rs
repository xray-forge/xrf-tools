use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_translation::TranslationProjectMode;
use xrf_utils::error_to_string;
use xrf_vfs::XrayRoots;

/// Report which layout roots look like, for the open form to preselect; on the pool, since it looks through the roots.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "detect_mode")]
pub async fn translations_detect_mode(
  roots: XrayRoots,
  execution: State<'_, ExecutionState>,
) -> TauriResult<TranslationProjectMode> {
  execution
    .run_blocking("Detecting a translations layout", move || {
      let mode: TranslationProjectMode = TranslationProjectMode::detect(&roots).map_err(error_to_string)?;

      log::info!(
        "Detected translations layout across {} root(s): {:?}",
        roots.roots.len(),
        mode
      );

      Ok(mode)
    })
    .await?
}
