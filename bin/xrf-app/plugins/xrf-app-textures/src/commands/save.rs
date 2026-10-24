use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobKind, JobRegistration, JobRegistry, JobStart, run_job};
use xrf_app_core::session::SessionSnapshot;
use xrf_app_core::types::TauriResult;
use xrf_app_render::texture_reloads::TextureReloads;
use xrf_job::{JobHandle, JobProgress};
use xrf_utils::error_to_string;

use crate::encoding::TextureEncodingSession;
use crate::request::TexturesSaveRequest;
use crate::save::{TextureSaveOutcome, write_save};
use crate::state::TextureState;

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
  reloads: State<'_, TextureReloads>,
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
    .map(|save| state.comparisons.require(save.session_id))
    .transpose()?;

  let (job, registration): (JobHandle, JobRegistration) =
    registry.register(start.with_resources(request.to_resources()).with_progress(progress))?;
  let assets: AssetMountState = AssetMountState::clone(&state.assets);

  let outcome: TextureSaveOutcome = run_job(
    &execution,
    "Texture save",
    registration,
    move || {
      let texture_bytes: Option<Vec<u8>> = match (&request.texture, comparison) {
        (Some(save), Some(held)) => Some(held.require(save.format)?.write_to_bytes().map_err(error_to_string)?),
        _ => None,
      };

      write_save(&assets, &job, &request, texture_bytes)
    },
    |outcome| outcome.outcome,
  )
  .await?;

  // Every viewport drawing what was written draws it as written.
  reloads.reload_written(&outcome.written);

  Ok(outcome)
}
