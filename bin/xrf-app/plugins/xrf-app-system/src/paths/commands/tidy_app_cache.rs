use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::cache::{AppCache, AppCachePolicy, AppCacheTidy};
use xrf_app_core::execution::ExecutionState;

/// Tidies the application's caches by `policy`, the settings' limits: what went unused past its age goes, then the
/// least recently used past its size; once a day as the page starts, or now where `is_forced`.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "tidy_app_cache")]
pub async fn system_tidy_app_cache(
  cache: State<'_, AppCache>,
  execution: State<'_, ExecutionState>,
  policy: AppCachePolicy,
  is_forced: bool,
) -> TauriResult<AppCacheTidy> {
  let cache: AppCache = cache.inner().clone();

  // Every entry is listed, and some removed, which grows with the caches.
  execution
    .run_blocking("Tidying the application's caches", move || {
      cache.tidy(policy, is_forced)
    })
    .await
}
