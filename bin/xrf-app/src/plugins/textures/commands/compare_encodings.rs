use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_dds::{DdsEncodeAttempt, DdsEncodeCandidate, DdsFile, DdsMetadata, DdsMipChain, DdsMipmaps, RgbaImage};
use xrf_job::{JobHandle, JobOutcome, JobProgress, JobScope};
use xrf_utils::format_path;
use xrf_vfs::XrayAssetType;

use crate::core::assets::{AssetMountState, read_referenced_asset};
use crate::core::error::error_to_string;
use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobKind, JobRegistration, JobRegistry, JobStart, run_job};
use crate::core::session::SessionId;
use crate::core::types::TauriResult;
use crate::plugins::textures::encoding::{TextureEncodingComparison, TextureEncodingCurrent, TextureEncodingSession};
use crate::plugins::textures::lease::{TEXTURE_ENCODE_GROUP, TEXTURE_PHASE_WEIGH};
use crate::plugins::textures::request::TexturesCompareRequest;
use crate::plugins::textures::source::TextureSource;
use crate::plugins::textures::state::TextureState;

/// What one run weighed, before the encodes and the figures are told apart.
struct MeasuredComparison {
  session: TextureEncodingSession,
  current: TextureEncodingCurrent,
  outcome: JobOutcome,
}

/// Weigh every candidate format against one texture, and keep the encodes.
///
/// The source is decoded once and reduced once; the candidates are then encoded from those same levels, so the
/// comparison weighs formats rather than weighing one format against a differently built chain. Every figure is
/// relative to the current file as decoded, which for a texture already stored in a DXT family is itself lossy - so
/// what is reported is the loss a re-encode adds, never distance from an original.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "compare_encodings"))]
#[tauri::command(rename = "compare_encodings")]
pub async fn textures_compare_encodings(
  request: TexturesCompareRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
  state: State<'_, TextureState>,
  assets: State<'_, AssetMountState>,
  registry: State<'_, Arc<JobRegistry>>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<TextureEncodingComparison> {
  let start: JobStart = JobStart::new(job_id, JobKind::TexturesCompareEncodings).with_request(&request);

  log::info!("Comparing encodings of texture: {}", request.source.label());

  let mipmaps: DdsMipmaps = request.to_mipmaps()?;

  let (job, registration): (JobHandle, JobRegistration) =
    registry.register(start.with_exclusion_group(TEXTURE_ENCODE_GROUP).with_progress(progress))?;

  let session_id: SessionId = request.session_id;

  state.comparison.begin_open(session_id)?;

  let state: TextureState = TextureState::clone(&state);
  let assets: AssetMountState = AssetMountState::clone(&assets);

  run_job(
    &execution,
    "Texture encoding comparison",
    registration,
    move || -> TauriResult<TextureEncodingComparison> {
      let bytes: Vec<u8> = read_texture_bytes(&assets, &request)?;
      let measured: MeasuredComparison = measure(&job, &bytes, &request, session_id, mipmaps)?;
      let comparison: TextureEncodingComparison = measured.session.to_comparison(measured.current, measured.outcome);

      // Completed candidates from a cancelled run remain usable while this session is current.
      state.comparison.commit_open(session_id, measured.session)?;

      log::info!(
        "Compared {} encodings of '{}'",
        comparison.candidates.len(),
        comparison.reference
      );

      Ok(comparison)
    },
    |comparison| comparison.outcome,
  )
  .await
}

/// The stored bytes of whatever the request names.
///
/// Two doors, because a texture has two kinds of address, and which door to take is what the source says rather than
/// what can be derived from it. A file names its own bytes, whether or not a tree could place it; a reference names a
/// texture without saying where it lives, so the roots decide which of several wins and an archived entry answers as
/// readily as a loose one.
fn read_texture_bytes(assets: &AssetMountState, request: &TexturesCompareRequest) -> TauriResult<Vec<u8>> {
  match &request.source {
    TextureSource::File { .. } => {
      // The texture beside what was named rather than the named file itself: a person opens a texture by picking
      // either half of the pair, and weighing the descriptor's own bytes reads a chunked file as a dds.
      let path: PathBuf = request
        .source
        .to_texture_path()
        .ok_or_else(|| String::from("A texture outside every root has to be named by a file path"))?;

      std::fs::read(&path).map_err(|error| format!("Cannot read '{}': {error}", format_path(&path)))
    }
    TextureSource::Asset { reference } => assets
      .with_probe(&request.roots, |probe| {
        read_referenced_asset(probe, XrayAssetType::Dds, reference)
      })?
      .map_err(error_to_string),
  }
}

/// Decode the texture once, reduce it once, and encode every candidate from those levels.
fn measure(
  job: &JobHandle,
  bytes: &[u8],
  request: &TexturesCompareRequest,
  session_id: SessionId,
  mipmaps: DdsMipmaps,
) -> TauriResult<MeasuredComparison> {
  let file: DdsFile = DdsFile::read_from_bytes(bytes).map_err(error_to_string)?;
  let metadata: DdsMetadata = file.metadata();
  let base: RgbaImage = file.decode_rgba(0).map_err(error_to_string)?;
  let chain: DdsMipChain = DdsMipChain::build(&base, mipmaps).map_err(error_to_string)?;

  let weighing: JobScope = job.enter(TEXTURE_PHASE_WEIGH, Some(DdsEncodeCandidate::ALL.len() as u64));
  let mut attempts: Vec<DdsEncodeAttempt> = Vec::with_capacity(DdsEncodeCandidate::ALL.len());
  let mut outcome: JobOutcome = JobOutcome::Completed;

  for candidate in DdsEncodeCandidate::ALL {
    if job.is_cancelled() {
      outcome = JobOutcome::Cancelled;
      break;
    }

    job.set_detail(Some(candidate.label().to_owned()));
    attempts.push(DdsEncodeAttempt::measure(&chain, candidate, request.quality.to_quality()).map_err(error_to_string)?);
    weighing.advance();
  }

  job.set_detail(None);

  Ok(MeasuredComparison {
    session: TextureEncodingSession {
      session_id,
      source: request.source.clone(),
      roots: request.roots.clone(),
      label: request.source.to_label(),
      attempts,
    },
    current: TextureEncodingCurrent {
      label: metadata.get_format_label(),
      file_bytes: metadata.file_size,
      gpu_bytes: metadata.file_size.saturating_sub(metadata.metadata_size),
      width: metadata.width,
      height: metadata.height,
      mipmap_levels: metadata.mipmap_levels,
    },
    outcome,
  })
}
