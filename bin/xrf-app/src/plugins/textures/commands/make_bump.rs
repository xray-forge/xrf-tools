use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;
use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobOutcome, JobProgress};
use xrf_texture::{GenerateBumpGloss, GenerateBumpOptions, GenerateBumpProcessor, read_image_as_rgba};
use xrf_utils::to_portable_path_string;

use crate::core::error::error_to_string;
use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart, run_job};
use crate::core::types::TauriResult;
use crate::plugins::textures::lease::{MAKE_BUMP_JOB_KIND, TEXTURE_ENCODE_GROUP, to_texture_lease_key};
use crate::plugins::textures::request::TexturesMakeBumpRequest;

/// What a generated pair came to.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureMakeBumpOutcome {
  /// Whether the pair was written or the run stopped because it was asked to.
  ///
  /// A cancelled run wrote neither half: both are encoded before either is written, so there is no point at which
  /// stopping could leave one half of a pair on disk with the other missing.
  pub outcome: JobOutcome,
  /// The normals and gloss, written as `<name>_bump.dds`.
  pub bump: String,
  /// The compression error and the height, written as `<name>_bump#.dds`.
  pub companion: String,
  /// Mean gloss over the whole surface, in `0..=1`.
  pub gloss_power: f32,
  /// Whether the gloss is too dark for the surface to show a specular response worth having.
  ///
  /// A verdict rather than a failure, exactly as in the SDK: the pair is written either way, because a modder who
  /// meant to author a matte surface is not making a mistake and one who did not wants to be told.
  pub is_gloss_too_dark: bool,
}

/// Generate the `_bump` and `_bump#` pair a bumped surface binds, from a height map.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "make_bump"))]
#[tauri::command(rename = "make_bump")]
pub async fn textures_make_bump(
  request: TexturesMakeBumpRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
  registry: State<'_, Arc<JobRegistry>>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<TextureMakeBumpOutcome> {
  log::info!("Generating bump pair for: {}", request.destination);

  let mut options: GenerateBumpOptions = to_options(&request)?;
  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, MAKE_BUMP_JOB_KIND)
      .with_exclusion_group(TEXTURE_ENCODE_GROUP)
      .with_lease_keys(to_pair_lease_keys(&options))
      .with_request(&request)
      .with_progress(progress),
  )?;

  options.job = job;

  run_job(
    &execution,
    "Bump pair generation",
    registration,
    move || {
      GenerateBumpProcessor::generate(&options).map(|result| TextureMakeBumpOutcome {
        outcome: result.outcome,
        bump: to_portable_path_string(&result.bump),
        companion: to_portable_path_string(&result.companion),
        gloss_power: result.gloss_power,
        is_gloss_too_dark: result.is_gloss_too_dark(),
      })
    },
    |outcome| outcome.outcome,
  )
  .await
}

/// Both halves the run would write, as lease keys.
///
/// Taken from the options rather than from the request, because the two paths are the generator's own naming rule and
/// a second spelling of it here would be a second place for `_bump#` to be got wrong.
fn to_pair_lease_keys(options: &GenerateBumpOptions) -> Vec<String> {
  GenerateBumpProcessor::pair_paths(&options.destination)
    .iter()
    .map(|path| to_texture_lease_key(path))
    .collect()
}

/// Read every image the request names and settle the gloss it asks for.
fn to_options(request: &TexturesMakeBumpRequest) -> TauriResult<GenerateBumpOptions> {
  if !(0.0..=1.0).contains(&request.gloss_constant) {
    return Err(format!(
      "Gloss level {} is outside the 0 to 1 range a gloss is measured in",
      request.gloss_constant
    ));
  }

  Ok(GenerateBumpOptions {
    // Replaced by the registered handle once the job starts; inert until then so nothing reports into a job that
    // does not exist yet.
    job: JobHandle::inert(),
    destination: PathBuf::from(&request.destination),
    height: read_image_as_rgba(&request.height).map_err(error_to_string)?,
    gloss: match &request.gloss {
      Some(path) => GenerateBumpGloss::Mask(read_image_as_rgba(path).map_err(error_to_string)?),
      None => GenerateBumpGloss::Constant(request.gloss_constant),
    },
    normal_map: match &request.normal_map {
      Some(path) => Some(read_image_as_rgba(path).map_err(error_to_string)?),
      None => None,
    },
    virtual_height: request.virtual_height,
    mip_filter: request.to_mip_filter()?,
    quality: request.quality.to_quality(),
  })
}
