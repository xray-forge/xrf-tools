use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobStart};
use xrf_app_render::TextureReloads;
use xrf_dds::DdsMipFilter;
use xrf_job::JobProgress;
use xrf_texture::GenerateBumpProcessor;
use xrf_utils::error_to_string;

use crate::edit::{TextureMakeBumpOutcome, TexturesMakeBumpRequest};
use crate::ipc::textures::TEXTURES_JOB_MAKE_BUMP;
use crate::lease::TEXTURE_ENCODE_GROUP;

/// Generate the `_bump` and `_bump#` pair a bumped surface binds, from a height map.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "make_bump"))]
#[tauri::command(rename = "make_bump")]
pub async fn textures_make_bump(
  request: TexturesMakeBumpRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
  registry: State<'_, Arc<JobRegistry>>,
  execution: State<'_, ExecutionState>,
  reloads: State<'_, TextureReloads>,
) -> TauriResult<TextureMakeBumpOutcome> {
  log::info!("Generating bump pair for: {}", request.destination);

  if !(0.0..=1.0).contains(&request.gloss_constant) {
    return Err(format!(
      "Gloss level {} is outside the 0 to 1 range a gloss is measured in",
      request.gloss_constant
    ));
  }

  let mip_filter: DdsMipFilter = request.to_mip_filter()?;
  let start: JobStart = JobStart::new(job_id, TEXTURES_JOB_MAKE_BUMP)
    .with_request(&request)
    .with_exclusion_group(TEXTURE_ENCODE_GROUP)
    .with_resources(request.to_resources())
    .with_progress(progress);

  let outcome: TextureMakeBumpOutcome = registry
    .run(
      &execution,
      start,
      move |job| {
        GenerateBumpProcessor::generate(&request.to_options(job, mip_filter)?)
          .map_err(error_to_string)
          .map(|result| TextureMakeBumpOutcome::of(&result))
      },
      |outcome| outcome.outcome,
    )
    .await?;

  // Every viewport drawing what was written draws it as written.
  reloads.reload_written(&[&outcome.bump, &outcome.companion]);

  Ok(outcome)
}
