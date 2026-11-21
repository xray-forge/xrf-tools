use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::JobRegistry;
use xrf_job::JobProgress;

use crate::conversion::{SpawnConversion, SpawnConversionResult};
use crate::request::SpawnConvertRequest;

/// Pack a directory of spawn chunks into one file, as a tracked job sharing one lane with unpacking; refused where the
/// file is already there, unless forced.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "pack_file")]
pub(crate) async fn spawn_pack_file(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: SpawnConvertRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<SpawnConversionResult> {
  SpawnConversion::Pack
    .run(&execution, &registry, request, job_id, progress)
    .await
}
