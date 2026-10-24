use std::sync::Arc;

use tauri::State;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionParts;
use xrf_app_core::types::TauriResult;
use xrf_vfs::XrayRevision;

use crate::descriptor::ConfigsProjectDescriptor;
use crate::opened_configs_project::OpenedConfigsProject;
use crate::request::ConfigsOpenRequest;
use crate::state::ConfigsState;

/// Opens a configs project for browsing, and lists what it holds.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_project"))]
#[tauri::command(rename = "open_project")]
pub async fn configs_open_project(
  execution: State<'_, ExecutionState>,
  state: State<'_, ConfigsState>,
  assets: State<'_, AssetMountState>,
  request: ConfigsOpenRequest,
) -> TauriResult<Arc<ConfigsProjectDescriptor>> {
  let ConfigsOpenRequest {
    session_id,
    roots,
    prefix,
    is_dltx,
  } = request;

  log::info!("Opening ltx configs project in {}", roots.describe());

  state.begin_open(session_id)?;
  let state: ConfigsState = state.inner().clone();
  let reader: AssetMountState = AssetMountState::clone(&assets);

  let opened: OpenedConfigsProject = execution
    .run_blocking("Configs project open", move || {
      OpenedConfigsProject::read(&reader.search_fresh(&roots)?, roots, prefix, is_dltx)
    })
    .await??;
  let revision: XrayRevision = opened.get_revision();
  let (project, inputs) = opened.into_project(session_id);
  let descriptor: Arc<ConfigsProjectDescriptor> = Arc::clone(&project.descriptor);

  state.catch_up(
    state.commit_open_with(session_id, project, revision, inputs, SessionParts::new())?,
    &assets,
  );

  Ok(descriptor)
}
