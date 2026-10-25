use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobStart};
use xrf_app_render::TextureReloads;
use xrf_job::JobProgress;
use xrf_texture::{BuildTextureOptions, read_image_as_rgba};
use xrf_thm::ThmFile;
use xrf_utils::error_to_string;

use crate::edit::{TextureBuildOutcome, TexturesBuildRequest};
use crate::ipc::textures::TEXTURES_JOB_BUILD;
use crate::lease::TEXTURE_ENCODE_GROUP;

/// Rebuild a texture from a source image, the way its descriptor says to.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "build_from_source")]
pub async fn textures_build_from_source(
  request: TexturesBuildRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
  registry: State<'_, Arc<JobRegistry>>,
  execution: State<'_, ExecutionState>,
  reloads: State<'_, TextureReloads>,
) -> TauriResult<TextureBuildOutcome> {
  let start: JobStart = JobStart::new(job_id, TEXTURES_JOB_BUILD)
    .with_request(&request)
    .with_exclusion_group(TEXTURE_ENCODE_GROUP)
    .with_resources(request.to_resources())
    .with_progress(progress);

  log::info!("Building texture {} from {}", request.destination, request.source);

  let outcome: TextureBuildOutcome = registry
    .run(
      &execution,
      start,
      move |job| {
        let options: BuildTextureOptions = BuildTextureOptions {
          destination: request.to_destination(),
          source: read_image_as_rgba(&request.source).map_err(error_to_string)?,
          descriptor: request.descriptor.to_descriptor(Some(ThmFile::new_texture())),
          quality: request.quality.to_quality(),
        };

        TextureBuildOutcome::build(&job, &options)
      },
      |outcome| outcome.outcome,
    )
    .await?;

  // Every viewport drawing what was written draws it as written.
  reloads.reload_written(&[&outcome.destination]);

  Ok(outcome)
}
