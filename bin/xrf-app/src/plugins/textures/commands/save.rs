use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};

use crate::core::error::error_to_string;
use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart, run_job};
use crate::core::types::TauriResult;
use crate::plugins::textures::lease::SAVE_JOB_KIND;
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
  log::info!(
    "Saving texture node: descriptor {}, texture {}",
    request.descriptor.is_some(),
    request.texture.is_some()
  );

  // Serialized before the job is registered, so a save that cannot claim the bytes it was asked to write fails here
  // rather than after taking a lease on the files.
  let texture_bytes: Option<Vec<u8>> = match &request.texture {
    Some(save) => Some(state.with_held_encoding(save.format, |file| file.write_to_bytes().map_err(error_to_string))?),
    None => None,
  };

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, SAVE_JOB_KIND)
      .with_lease_keys(request.to_lease_keys())
      .with_request(&request)
      .with_progress(progress),
  )?;

  run_job(
    &execution,
    "Texture save",
    registration,
    move || write_save(&job, &request, texture_bytes),
    |outcome| outcome.outcome,
  )
  .await
}
