use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::JobRegistry;
use xrf_job::JobProgress;

use crate::request::SpawnConversionRequest;
use crate::{SpawnConversion, SpawnConversionResult};

/// Unpack a spawn file into a directory of chunks, as a tracked job sharing one lane with packing.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "unpack_file"))]
#[tauri::command(rename = "unpack_file")]
pub async fn spawn_unpack_file(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: SpawnConversionRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<SpawnConversionResult> {
  SpawnConversion::Unpack
    .run(&execution, &registry, request, job_id, progress)
    .await
}
