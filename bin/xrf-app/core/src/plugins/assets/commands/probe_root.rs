use xrf_vfs::XrayRootProbe;

/// Describe what a path is, without mounting it.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "probe_root"))]
#[tauri::command(rename = "probe_root")]
pub async fn assets_probe_root(path: String) -> XrayRootProbe {
  let probe: XrayRootProbe = XrayRootProbe::describe(&path);

  log::info!("Probed root {path}: {:?} with {} mount(s)", probe.kind, probe.mounts);

  probe
}
