use std::sync::Arc;

use tauri::State;
use xrf_dltx::select_ltx_dialect;
use xrf_environment::{
  EnvironmentCatalog, EnvironmentFinding, EnvironmentReadOptions, EnvironmentReader, WeatherCycleId,
};
use xrf_ltx::LtxProject;

use crate::core::assets::{AssetMountState, detect_roots_engine};
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
  cycle: WeatherCycleId,
  state: State<'_, EnvironmentState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<EnvironmentCycleDescription> {
  let held: Option<Arc<EnvironmentCatalog>> = state.get(&request);
  let read: EnvironmentRequest = request.clone();
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let (description, catalog) = execution
    .run_blocking("Reading a weather cycle", move || {
      let project: LtxProject = open_configs(&read.roots, select_ltx_dialect(read.is_dltx))?;
      // Findings about what the cycle names are the whole catalog's to find, so it is read where none is held.
      let options: EnvironmentReadOptions = EnvironmentReadOptions::default()
        .with_engine(read.engine.resolve(|| detect_roots_engine(&assets, &read.roots)).engine);
      let catalog: Arc<EnvironmentCatalog> = match held {
        Some(catalog) => catalog,
        None => Arc::new(read_catalog(&project, &options)?),
      };
      let (authored, _) = EnvironmentReader::read_cycle(&project, &cycle, &options.with_explained(true))
        .map_err(|error| {
          format!(
            "Failed to read {} '{}': {error}",
            cycle.kind.get_subject().to_lowercase(),
            cycle.name
          )
        })?
        .ok_or_else(|| {
          format!(
            "There is no {} '{}'",
            cycle.kind.get_subject().to_lowercase(),
            cycle.name
          )
        })?;
      let findings: Vec<EnvironmentFinding> = catalog
        .findings
        .iter()
        .filter(|finding| finding.file == authored.file)
        .cloned()
        .collect();

      TauriResult::Ok((
        EnvironmentCycleDescription {
          cycle: authored,
          findings,
        },
        catalog,
      ))
    })
    .await??;

  state.put(request, catalog);

  Ok(description)
}
