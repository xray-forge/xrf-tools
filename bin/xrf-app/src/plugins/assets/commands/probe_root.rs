use xrf_vfs::XrayRootProbe;

/// Describe what a path is, without mounting it.
///
/// Answers the question a path setting asks and planning alone cannot: [`xrf_vfs::XrayMountMode::Auto`] plans any
/// readable directory as a root, so a source repository and a game data tree plan identically. The probe carries the
/// evidence that separates them, so a surface can say a directory holds nothing an engine would load.
///
/// Cannot fail — an unreadable or absent path is one of the answers rather than an error.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "probe_root"))]
#[tauri::command(rename = "probe_root")]
pub async fn assets_probe_root(path: String) -> XrayRootProbe {
  let probe: XrayRootProbe = XrayRootProbe::describe(&path);

  log::info!("Probed root {path}: {:?} with {} mount(s)", probe.kind, probe.mounts);

  probe
}
