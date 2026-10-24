use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobKind, JobRegistration, JobRegistry, JobResource, JobStart, run_job};
use xrf_app_core::types::TauriResult;
use xrf_error::XrfError;
use xrf_job::{JobHandle, JobProgress};
use xrf_ltx::{LtxFormatOptions, LtxProject, LtxProjectFormatResult};
use xrf_utils::error_to_string;
use xrf_vfs::{XrayRoots, XraySearch, XrayTransaction};

use crate::request::ConfigsFormatRequest;

/// Rewrite the LTX configs roots exposes.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "format_directory"))]
#[tauri::command(rename = "format_directory")]
pub async fn configs_format_directory(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  assets: State<'_, AssetMountState>,
  request: ConfigsFormatRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<LtxProjectFormatResult> {
  let start: JobStart = JobStart::new(job_id, JobKind::ConfigsFormat).with_request(&request);

  let ConfigsFormatRequest { roots, prefix } = request;

  log::info!("Formatting ltx configs in {}", roots.describe());

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    start
      .with_exclusion_group(JobKind::ConfigsFormat.as_str())
      .with_resources(roots.roots.iter().map(|root| JobResource::tree(&root.path)).collect())
      .with_progress(progress),
  )?;
  let assets: AssetMountState = AssetMountState::clone(&assets);

  run_job(
    &execution,
    "Configs formatting",
    registration,
    move || {
      format_directory(
        &assets,
        &roots,
        prefix.as_deref(),
        LtxFormatOptions::default().with_job(job),
      )
    },
    |result| result.outcome,
  )
  .await
}

/// Formats the configs `roots` hold below `prefix` and commits every rewrite as one change of the shared snapshot, all
/// or none, so the cache, sessions and pages follow it as any save. The configs are read through a mount of the
/// format's own, walked afresh, so one created behind the app is formatted too.
///
/// # Errors
///
/// Returns an error, with nothing written, when the roots cannot be mounted, a config is archived, cannot be read or
/// will not parse, or the rewrite cannot be committed.
pub(crate) fn format_directory(
  assets: &AssetMountState,
  roots: &XrayRoots,
  prefix: Option<&str>,
  options: LtxFormatOptions,
) -> TauriResult<LtxProjectFormatResult> {
  // Opened in the shared snapshot too, so every rewrite lands in a mount it holds.
  let search: XraySearch = assets.search_fresh(roots)?;
  let project: LtxProject =
    LtxProject::open_at_roots_opt(roots, prefix, Default::default()).map_err(error_to_string)?;

  project
    .format_all_files_with(options, |rewrites| {
      let transaction: XrayTransaction = rewrites
        .into_iter()
        .fold(XrayTransaction::new(&search), |transaction, (file, bytes)| {
          transaction.with_file_write(file, bytes)
        });

      assets.commit(&transaction).map(drop).map_err(XrfError::new_asset_error)
    })
    .map_err(error_to_string)
}
