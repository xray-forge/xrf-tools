use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::JobRegistry;
use xrf_app_core::types::TauriResult;
use xrf_job::JobProgress;

use crate::conversion::{SpawnConversion, SpawnConversionResult, run_conversion};
use crate::request::SpawnConversionRequest;

/// Pack a spawn file as an exclusive, tracked background job.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "pack_file"))]
#[tauri::command(rename = "pack_file")]
pub async fn spawn_pack_file(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: SpawnConversionRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<SpawnConversionResult> {
  run_conversion(&execution, &registry, request, SpawnConversion::Pack, job_id, progress).await
}
