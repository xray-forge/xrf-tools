use std::sync::Arc;

use tauri::State;
use xrf_dltx::select_ltx_dialect;
use xrf_environment::{EnvironmentCatalog, EnvironmentReadOptions};
use xrf_ltx::LtxProject;

use crate::core::assets::{AssetMountState, detect_roots_engine};
use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::environment::catalog::{open_configs, read_catalog};
use crate::plugins::environment::description::EnvironmentCatalogDescription;
use crate::plugins::environment::request::EnvironmentRequest;
use crate::plugins::environment::state::EnvironmentState;

/// Read a game's environment configs afresh as one engine reads them, and list what they hold.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "read_catalog"))]
#[tauri::command(rename = "read_catalog")]
pub async fn environment_read_catalog(
  request: EnvironmentRequest,
  state: State<'_, EnvironmentState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<EnvironmentCatalogDescription> {
  let read: EnvironmentRequest = request.clone();
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let catalog: Arc<EnvironmentCatalog> = execution
    .run_blocking("Reading the environment configs", move || {
      let project: LtxProject = open_configs(&read.roots, select_ltx_dialect(read.is_dltx))?;

      read_catalog(
        &project,
        &EnvironmentReadOptions::default()
          .with_engine(read.engine.resolve(|| detect_roots_engine(&assets, &read.roots)).engine),
      )
      .map(Arc::new)
    })
    .await??;

  state.put(request, Arc::clone(&catalog));

  Ok(EnvironmentCatalogDescription::from(catalog.as_ref()))
}
