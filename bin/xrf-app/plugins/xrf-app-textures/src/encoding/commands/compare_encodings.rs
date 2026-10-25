use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobStart};
use xrf_app_core::session::{SessionId, SessionOpening};
use xrf_dds::DdsMipmaps;
use xrf_job::JobProgress;

use crate::encoding::{TextureComparison, TextureComparisons, TextureEncodingComparison, TexturesCompareRequest};
use crate::ipc::textures::TEXTURES_JOB_COMPARE_ENCODINGS;
use crate::lease::TEXTURE_ENCODE_GROUP;

/// Weigh every candidate format against one texture, and keep the encodes with the texture they were made of.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "compare_encodings")]
pub async fn textures_compare_encodings(
  request: TexturesCompareRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
  comparisons: State<'_, TextureComparisons>,
  assets: State<'_, AssetMountState>,
  registry: State<'_, Arc<JobRegistry>>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<TextureEncodingComparison> {
  log::info!("Comparing encodings of texture: {}", request.source.label());

  // Both refusals come before the job registers, so a refused comparison never shows as a failed job.
  let mipmaps: DdsMipmaps = request.to_mipmaps()?;
  let session_id: SessionId = request.session_id;
  let opening: SessionOpening<TextureComparison> = comparisons.begin_open(session_id)?;
  let start: JobStart = JobStart::new(job_id, TEXTURES_JOB_COMPARE_ENCODINGS)
    .with_request(&request)
    .with_exclusion_group(TEXTURE_ENCODE_GROUP)
    .with_progress(progress);
  let assets: AssetMountState = AssetMountState::clone(&assets);

  registry
    .run(
      &execution,
      start,
      move |job| -> TauriResult<TextureEncodingComparison> {
        let (bytes, files) = request.read_dds(&assets)?;
        let (comparison, current, outcome) = TextureComparison::measure(&job, &request, &bytes, files, mipmaps)?;
        let answer: TextureEncodingComparison = comparison.to_comparison(session_id, current, outcome);

        // Completed candidates from a cancelled run remain usable while this comparison is held.
        opening.commit(comparison)?;

        log::info!(
          "Compared {} encodings of '{}'",
          answer.candidates.len(),
          answer.reference
        );

        Ok(answer)
      },
      |comparison| comparison.outcome,
    )
    .await
}
