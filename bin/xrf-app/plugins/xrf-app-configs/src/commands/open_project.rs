use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionSnapshot;
use xrf_vfs::XrayRoots;

use crate::project::{ConfigsProject, ConfigsProjectDescriptor, ConfigsProjects, OpenedConfigsProject};
use crate::request::ConfigsOpenRequest;

/// Open a configs project for browsing over the shared asset roots, read on the pool, and list what it holds.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "open_project")]
pub async fn configs_open_project(
  execution: State<'_, ExecutionState>,
  projects: State<'_, ConfigsProjects>,
  assets: State<'_, AssetMountState>,
  request: ConfigsOpenRequest,
) -> TauriResult<SessionSnapshot<Arc<ConfigsProjectDescriptor>>> {
  let ConfigsOpenRequest {
    session_id,
    roots,
    prefix,
    is_dltx,
  } = request;

  log::info!("Opening ltx configs project in {}", roots.describe());

  let read_roots: XrayRoots = roots.clone();
  let opened: Arc<SessionSnapshot<ConfigsProject>> = projects
    .open_followed(
      &execution,
      &assets,
      session_id,
      roots,
      "Opening a configs project",
      move |search| OpenedConfigsProject::read(&search, read_roots, prefix, is_dltx),
    )
    .await?;

  Ok(opened.map(ConfigsProject::describe))
}
