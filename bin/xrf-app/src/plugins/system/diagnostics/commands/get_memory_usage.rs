use std::sync::mpsc::Receiver;
use std::time::Duration;

use tauri::State;
use xrf_renderer::RenderMemoryReport;

use crate::core::types::TauriResult;
use crate::plugins::render::render_answer::await_answer;
use crate::plugins::render::state::RenderState;
use crate::plugins::system::diagnostics::memory::{GpuMemoryUsage, MemoryUsage, WebviewProcess, WebviewProcessProbe};

/// How long the render thread is waited on for its GPU reading, which it gives between two frames.
const GPU_ANSWER_TIMEOUT: Duration = Duration::from_millis(500);

/// Report what the application and each of its webview's processes hold in memory, where the platform can say, and what
/// the application holds on the GPU while its renderer runs one.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_memory_usage"))]
#[tauri::command(rename = "get_memory_usage")]
pub async fn system_get_memory_usage(
  probe: State<'_, WebviewProcessProbe>,
  render: State<'_, RenderState>,
) -> TauriResult<Option<MemoryUsage>> {
  // Asked first, so the render thread answers while the webview's processes are listed.
  let answer: Receiver<Option<RenderMemoryReport>> = render.renderer.describe_memory();
  let processes: Vec<WebviewProcess> = probe.list().await?;
  // No render thread lets the reply go, and a busy one is not waited on past the timeout: both read as no GPU.
  let report: Option<RenderMemoryReport> =
    await_answer(answer, GPU_ANSWER_TIMEOUT, String::from("The renderer's GPU memory"))
      .await
      .ok()
      .flatten();

  Ok(MemoryUsage::read(
    std::process::id(),
    &processes,
    report.as_ref().map(GpuMemoryUsage::from),
  ))
}
