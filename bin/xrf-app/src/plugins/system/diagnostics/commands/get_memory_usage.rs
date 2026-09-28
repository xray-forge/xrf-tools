use tauri::State;

use crate::core::types::TauriResult;
use crate::plugins::system::diagnostics::memory::{MemoryUsage, WebviewProcess, WebviewProcessProbe};

/// Report what the application and each of its webview's processes hold in memory, where the platform can say.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_memory_usage"))]
#[tauri::command(rename = "get_memory_usage")]
pub async fn system_get_memory_usage(probe: State<'_, WebviewProcessProbe>) -> TauriResult<Option<MemoryUsage>> {
  let processes: Vec<WebviewProcess> = probe.list().await?;

  Ok(MemoryUsage::read(std::process::id(), &processes))
}
