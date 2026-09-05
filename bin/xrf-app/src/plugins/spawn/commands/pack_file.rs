use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::JobProgress;

use crate::core::execution::ExecutionState;
use crate::core::jobs::JobRegistry;
use crate::core::types::TauriResult;
use crate::plugins::spawn::conversion::{SpawnConversion, SpawnConversionResult, run_conversion};
use crate::plugins::spawn::request::SpawnConversionRequest;

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
