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

use crate::edit::{TextureSaveAsOutcome, TextureSaveAsWriter, TextureWrite, TexturesSaveAsRequest};
use crate::encoding::{TextureComparison, TextureComparisons};
use crate::ipc::textures::TEXTURES_JOB_SAVE_AS;

/// Save a texture, as edited, under a new name: its texture, descriptor and bump pair, with a chosen re-encode as the
/// copy's pixels. The original is not changed.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "save_as")]
#[expect(
  clippy::too_many_arguments,
  reason = "Tauri injects each managed state as an argument of its own"
)]
pub(crate) async fn textures_save_as(
  request: TexturesSaveAsRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
  comparisons: State<'_, TextureComparisons>,
  assets: State<'_, AssetMountState>,
  registry: State<'_, Arc<JobRegistry>>,
  execution: State<'_, ExecutionState>,
  reloads: State<'_, TextureReloads>,
) -> TauriResult<TextureSaveAsOutcome> {
  log::info!(
    "Saving texture {} as {}",
    request.target.reference,
    request.target.destination
  );

  let held: Option<Arc<SessionSnapshot<TextureComparison>>> = request
    .texture
    .as_ref()
    .map(|save| comparisons.require(save.session_id))
    .transpose()?;
  // Checked before the job starts, so a re-encode of another texture is refused without a failed job.
  let texture_path: Option<PathBuf> = match &held {
    Some(held) => Some(
      held
        .require_paired(Path::new(&request.sources.descriptor.path))?
        .texture
        .clone(),
    ),
    None => None,
  };
  // Where the copy's files land is decided by the commit, so the job claims the original's files.
  let start: JobStart = JobStart::new(job_id, TEXTURES_JOB_SAVE_AS)
    .with_request(&request)
    .with_resources(vec![
      JobResource::file(&request.sources.texture.path),
      JobResource::file(&request.sources.descriptor.path),
    ])
    .with_progress(progress);
  let assets: AssetMountState = AssetMountState::clone(&assets);

  let outcome: TextureSaveAsOutcome = registry
    .run(
      &execution,
      start,
      move |job| {
        let texture: Option<TextureWrite> = match (&request.texture, held, texture_path) {
          (Some(save), Some(held), Some(path)) => Some(TextureWrite {
            bytes: held
              .require_for_save(save.format, request.form.has_alpha())?
              .write_to_bytes()
              .map_err(error_to_string)?,
            path,
            expected: save.expected,
            format: save.format,
          }),
          _ => None,
        };

        TextureSaveAsWriter::write(&assets, &job, &request, texture)
      },
      |outcome| outcome.outcome,
    )
    .await?;

  // Any viewport already showing one of the copy's names reloads it.
  reloads.reload_written(&outcome.written);

  Ok(outcome)
}
