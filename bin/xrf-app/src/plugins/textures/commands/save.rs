use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};

use crate::core::error::error_to_string;
use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobKind, JobRegistration, JobRegistry, JobStart, run_job};
use crate::core::session::SessionSnapshot;
use crate::core::types::TauriResult;
use crate::plugins::textures::encoding::TextureEncodingSession;
use crate::plugins::textures::request::TexturesSaveRequest;
use crate::plugins::textures::save::{TextureSaveOutcome, write_save};
use crate::plugins::textures::state::TextureState;

/// Write one node's pending files: its descriptor, its base texture, or both.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "save"))]
#[tauri::command(rename = "save")]
pub async fn textures_save(
  request: TexturesSaveRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
  state: State<'_, TextureState>,
  registry: State<'_, Arc<JobRegistry>>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<TextureSaveOutcome> {
  let start: JobStart = JobStart::new(job_id, JobKind::TexturesSave).with_request(&request);

  log::info!(
    "Saving texture node: descriptor {}, texture {}",
    request.descriptor.is_some(),
    request.texture.is_some()
  );

  let comparison: Option<Arc<SessionSnapshot<TextureEncodingSession>>> = request
    .texture
    .as_ref()
    .map(|save| state.comparison.require(save.session_id))
    .transpose()?;

  let (job, registration): (JobHandle, JobRegistration) =
    registry.register(start.with_resources(request.to_resources()).with_progress(progress))?;

  run_job(
    &execution,
    "Texture save",
    registration,
    move || {
      let texture_bytes: Option<Vec<u8>> = match (&request.texture, comparison) {
        (Some(save), Some(held)) => Some(held.require(save.format)?.write_to_bytes().map_err(error_to_string)?),
        _ => None,
      };

      write_save(&job, &request, texture_bytes)
    },
    |outcome| outcome.outcome,
  )
  .await
}
