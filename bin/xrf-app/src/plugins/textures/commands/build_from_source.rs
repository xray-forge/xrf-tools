use std::sync::Arc;

use serde::Serialize;
use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_db::ThmFile;
use xrf_job::{JobHandle, JobOutcome, JobProgress};
use xrf_texture::{BuildTextureOptions, BuildTextureProcessor, BuildTextureResult, read_image_as_rgba};
use xrf_utils::to_portable_path_string;

use crate::core::error::error_to_string;
use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart, run_job};
use crate::core::types::TauriResult;
use crate::plugins::textures::lease::{BUILD_JOB_KIND, TEXTURE_ENCODE_GROUP};
use crate::plugins::textures::request::TexturesBuildRequest;

/// One recipe field the descriptor asks for that this build does not carry out.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureBuildOmissionReport {
  /// The descriptor field this sits beside, under the name the SDK gives it.
  pub field: String,
  /// Why the build does not do it, in words a person deciding whether to rebuild can act on.
  pub reason: String,
}

/// What a rebuilt texture came to.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureBuildOutcome {
  /// Whether the texture was written or the run stopped before it started.
  ///
  /// A cancelled build wrote nothing. There is one boundary and it is before the work: the encode writes the file
  /// itself and is a single call with no seam inside it. That costs nothing worth having, because the encode is tens
  /// of milliseconds - a descriptor decides the format here and `ETFormat` has no name for BC7, so the one candidate
  /// that takes seconds cannot arise.
  pub outcome: JobOutcome,
  pub destination: String,
  /// Size of the source, which the descriptor's own width and height are refreshed from.
  pub width: u32,
  pub height: u32,
  /// Levels written, counting the base.
  pub mipmap_levels: u32,
  /// Recipe fields the descriptor asked for and the build did not carry out.
  pub omissions: Vec<TextureBuildOmissionReport>,
}

/// Rebuild a texture from a source image, the way its descriptor says to.
///
/// The descriptor decides everything - the layout, whether there is a mip chain and which kernel reduces it - and
/// nothing is taken from the file being replaced, so a rebuild is reproducible from the two inputs alone. A format the
/// build has no honest encoder for is refused rather than substituted; a field it merely does not implement yet comes
/// back in [`TextureBuildOutcome::omissions`].
///
/// Holds the file it would write, so a save or a generation aimed at it is refused rather than allowed to race it, and
/// joins the encode group for the same reason a generation does.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "build_from_source"))]
#[tauri::command(rename = "build_from_source")]
pub async fn textures_build_from_source(
  request: TexturesBuildRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
  registry: State<'_, Arc<JobRegistry>>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<TextureBuildOutcome> {
  log::info!("Building texture {} from {}", request.destination, request.source);

  let options: BuildTextureOptions = BuildTextureOptions {
    destination: request.to_destination(),
    source: read_image_as_rgba(&request.source).map_err(error_to_string)?,
    descriptor: request.descriptor.to_descriptor(Some(ThmFile::new_texture())),
    quality: request.quality.to_quality(),
  };

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, BUILD_JOB_KIND)
      .with_exclusion_group(TEXTURE_ENCODE_GROUP)
      .with_lease_keys(request.to_lease_keys())
      .with_request(&request)
      .with_progress(progress),
  )?;

  run_job(
    &execution,
    "Texture build",
    registration,
    move || build(&job, &options),
    |outcome| outcome.outcome,
  )
  .await
}

/// Build the texture, unless the run was already asked to stop.
fn build(job: &JobHandle, options: &BuildTextureOptions) -> TauriResult<TextureBuildOutcome> {
  // Read before the encode rather than after it. `BuildTextureProcessor::build` writes the file it encodes, so a
  // check on the far side would be reporting a stop for a run that had already published its result.
  if job.is_cancelled() {
    return Ok(TextureBuildOutcome {
      outcome: JobOutcome::Cancelled,
      destination: to_portable_path_string(&options.destination),
      width: 0,
      height: 0,
      mipmap_levels: 0,
      omissions: Vec::new(),
    });
  }

  let built: BuildTextureResult = BuildTextureProcessor::build(options).map_err(error_to_string)?;

  log::info!(
    "Built texture {}x{} with {} levels, {} recipe fields left out",
    built.width,
    built.height,
    built.mipmap_levels,
    built.omissions.len()
  );

  Ok(TextureBuildOutcome {
    outcome: JobOutcome::Completed,
    destination: to_portable_path_string(&built.destination),
    width: built.width,
    height: built.height,
    mipmap_levels: built.mipmap_levels,
    omissions: built
      .omissions
      .iter()
      .map(|omission| TextureBuildOmissionReport {
        field: omission.field().to_owned(),
        reason: omission.reason().to_owned(),
      })
      .collect(),
  })
}
