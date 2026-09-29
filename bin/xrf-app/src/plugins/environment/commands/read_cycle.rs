use std::sync::Arc;

use tauri::State;
use xrf_dltx::select_ltx_dialect;
use xrf_environment::{
  EnvironmentCatalog, EnvironmentFinding, EnvironmentReadOptions, EnvironmentReader, WeatherCycleKind,
};
use xrf_ltx::LtxProject;

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::environment::catalog::{open_configs, read_catalog};
use crate::plugins::environment::description::EnvironmentCycleDescription;
use crate::plugins::environment::request::EnvironmentRequest;
use crate::plugins::environment::state::EnvironmentState;

/// Read one cycle or effect as authored, recording where each value came from, with every finding in its config.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "read_cycle"))]
#[tauri::command(rename = "read_cycle")]
pub async fn environment_read_cycle(
  request: EnvironmentRequest,
  kind: WeatherCycleKind,
  name: String,
  state: State<'_, EnvironmentState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<EnvironmentCycleDescription> {
  let held: Option<Arc<EnvironmentCatalog>> = state.get(&request);
  let read: EnvironmentRequest = request.clone();
  let (description, catalog) = execution
    .run_blocking("Reading a weather cycle", move || {
      let project: LtxProject = open_configs(&read.roots, select_ltx_dialect(read.is_dltx))?;
      // Findings about what the cycle names are the whole catalog's to find, so it is read where none is held.
      let catalog: Arc<EnvironmentCatalog> = match held {
        Some(catalog) => catalog,
        None => Arc::new(read_catalog(&project, read.engine, &EnvironmentReadOptions::default())?),
      };
      let explained: EnvironmentReadOptions = EnvironmentReadOptions::default().with_explained(true);
      let (cycle, _) = EnvironmentReader::read_cycle(&project, read.engine, kind, &name, &explained)
        .map_err(|error| format!("Failed to read weather cycle '{name}': {error}"))?
        .ok_or_else(|| format!("There is no weather cycle '{name}'"))?;
      let findings: Vec<EnvironmentFinding> = catalog
        .findings
        .iter()
        .filter(|finding| finding.file == cycle.file)
        .cloned()
        .collect();

      TauriResult::Ok((EnvironmentCycleDescription { cycle, findings }, catalog))
    })
    .await??;

  state.put(request, catalog);

  Ok(description)
}
