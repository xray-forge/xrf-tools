use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::JobRegistry;
use xrf_job::JobProgress;

use crate::request::SpawnConvertRequest;
use crate::{SpawnConversion, SpawnConversionResult};

/// Pack a directory of spawn chunks into one file, as a tracked job sharing one lane with unpacking.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "pack_file")]
pub async fn spawn_pack_file(
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
