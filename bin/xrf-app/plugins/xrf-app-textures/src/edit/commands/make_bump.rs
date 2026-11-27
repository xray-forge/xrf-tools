use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobStart};
use xrf_app_render::TextureReloads;
use xrf_dds::DdsMipFilter;
use xrf_job::JobProgress;
use xrf_texture::{GenerateBumpProcessor, GenerateBumpResult};
use xrf_utils::error_to_string;

use crate::edit::{TextureMakeBumpOutcome, TextureStagedWriter, TexturesMakeBumpRequest};
use crate::ipc::textures::TEXTURES_JOB_MAKE_BUMP;
use crate::lease::TEXTURE_ENCODE_GROUP;

/// Generate the `_bump` and `_bump#` pair a bumped surface binds, from a height map.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "make_bump")]
pub(crate) async fn textures_make_bump(
  request: TexturesMakeBumpRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
  registry: State<'_, Arc<JobRegistry>>,
  execution: State<'_, ExecutionState>,
  reloads: State<'_, TextureReloads>,
  assets: State<'_, AssetMountState>,
) -> TauriResult<TextureMakeBumpOutcome> {
  log::info!("Generating bump pair for: {}", request.destination);

  let assets: AssetMountState = assets.inner().clone();
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
        let mut writer: TextureStagedWriter = TextureStagedWriter::default();
        let result: GenerateBumpResult =
          GenerateBumpProcessor::generate_into(&request.to_options(job, mip_filter)?, &mut writer)
            .map_err(error_to_string)?;

        // Both halves as one write, so the pair lands whole or not at all; a stopped run staged neither.
        writer.publish(&assets)?;

        TauriResult::Ok(TextureMakeBumpOutcome::of(&result))
      },
      |outcome| outcome.outcome,
    )
    .await?;

  // A file outside every watched root is drawn as written only once asked for; the watcher reloads the rest.
  reloads.reload_written(&[&outcome.bump, &outcome.companion]);

  Ok(outcome)
}
