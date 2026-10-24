use std::path::{Path, PathBuf};
use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobResource, JobStart};
use xrf_app_core::session::SessionSnapshot;
use xrf_app_render::TextureReloads;
use xrf_job::JobProgress;
use xrf_utils::error_to_string;

use crate::edit::{TextureSaveOutcome, TextureSaver, TextureWrite, TexturesSaveRequest};
use crate::encoding::{TextureComparison, TextureComparisons};
use crate::ipc::textures::TEXTURES_JOB_SAVE;

/// Write one node's descriptor, and the base texture with it when a held candidate is chosen: written over the texture
/// that candidate's comparison was made of, whose descriptor the request has to name.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "save"))]
#[tauri::command(rename = "save")]
// Tauri injects each managed state as an argument of its own.
#[allow(clippy::too_many_arguments)]
pub async fn textures_save(
  request: TexturesSaveRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
  comparisons: State<'_, TextureComparisons>,
  assets: State<'_, AssetMountState>,
  registry: State<'_, Arc<JobRegistry>>,
  execution: State<'_, ExecutionState>,
  reloads: State<'_, TextureReloads>,
) -> TauriResult<TextureSaveOutcome> {
  log::info!(
    "Saving texture node: descriptor {}, texture {}",
    request.descriptor.target.path,
    request.texture.is_some()
  );

  let held: Option<Arc<SessionSnapshot<TextureComparison>>> = request
    .texture
    .as_ref()
    .map(|save| comparisons.require(save.session_id))
    .transpose()?;
  // Settled before the job starts, so a refused pairing never shows as a failed job.
  let texture_path: Option<PathBuf> = match &held {
    Some(held) => Some(
      held
        .require_paired(Path::new(&request.descriptor.target.path))?
        .texture
        .clone(),
    ),
    None => None,
  };
  let start: JobStart = JobStart::new(job_id, TEXTURES_JOB_SAVE)
    .with_request(&request)
    .with_resources(
      std::iter::once(JobResource::file(&request.descriptor.target.path))
        .chain(texture_path.iter().map(JobResource::file))
        .collect(),
    )
    .with_progress(progress);
  let assets: AssetMountState = AssetMountState::clone(&assets);

  let outcome: TextureSaveOutcome = registry
    .run(
      &execution,
      start,
      move |job| {
        let texture: Option<TextureWrite> = match (request.texture, held, texture_path) {
          (Some(save), Some(held), Some(path)) => Some(TextureWrite {
            bytes: held.require(save.format)?.write_to_bytes().map_err(error_to_string)?,
            path,
            expected: save.expected,
            format: save.format,
          }),
          _ => None,
        };

        TextureSaver::save(&assets, &job, &request.descriptor, texture)
      },
      |outcome| outcome.outcome,
    )
    .await?;

  // Every viewport drawing what was written draws it as written.
  reloads.reload_written(&outcome.written);

  Ok(outcome)
}
