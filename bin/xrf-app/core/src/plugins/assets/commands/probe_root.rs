use tauri::State;
use xrf_vfs::XrayRootProbe;

use crate::TauriResult;
use crate::execution::ExecutionState;

/// Describe what a path is, without mounting it; on the pool, since a probe scans the directory.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "probe_root"))]
#[tauri::command(rename = "probe_root")]
pub async fn assets_probe_root(path: String, execution: State<'_, ExecutionState>) -> TauriResult<XrayRootProbe> {
  execution
    .run_blocking("Probing a root", move || {
      let probe: XrayRootProbe = XrayRootProbe::describe(&path);

      log::info!("Probed root {path}: {:?} with {} mount(s)", probe.kind, probe.mounts);

      probe
    })
    .await
}
