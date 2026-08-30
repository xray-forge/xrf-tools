use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::JobProgress;

use crate::core::jobs::JobRegistry;

/// Watch a running job this window did not start.
///
/// What makes a reload recoverable: the run kept going, but the channel it was reporting to belonged to the page that
/// went away, so a new page hands it one of its own. Without this a reloaded window can only ask the listing where the
/// job has got to, which is both slower than the job reports and noisy — every snapshot the old channel still receives
/// is a callback the webview cannot find.
///
/// The newest attach is the one that reports. Two windows watching one job is not a case this application has, and the
/// listing still describes the run for anybody who did not attach.
///
/// Answers whether anything is now reporting to `progress`. `false` means the job is not running — it finished while
/// the page was loading, or it never started — and the listing is what describes it then.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "attach"))]
#[tauri::command(rename = "attach")]
pub fn jobs_attach(registry: State<'_, Arc<JobRegistry>>, id: Uuid, progress: Channel<JobProgress>) -> bool {
  log::info!("Attaching to job: {}", id);

  registry.attach(id, progress)
}
