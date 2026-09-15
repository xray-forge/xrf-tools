use std::sync::{Arc, Mutex, MutexGuard};

use xrf_vfs::{XrayProbe, XrayProbeStep, XrayRoots, XrayVfs};

use crate::core::types::TauriResult;

/// Every mounted source the application holds, searched through per-request probes.
///
/// One VFS for the process rather than one per roots, because mounting is indexed eagerly and idempotent per planned
/// path: a viewer stepping through fifty models under one root pays for one index instead of fifty. Callers never
/// receive the VFS itself, only a probe over the steps their spec asked for, so an unscoped lookup cannot silently span
/// two unrelated roots.
#[derive(Clone)]
pub struct AssetMountState {
  vfs: Arc<Mutex<XrayVfs>>,
}

impl AssetMountState {
  pub fn new() -> Self {
    Self {
      vfs: Arc::new(Mutex::new(XrayVfs::new())),
    }
  }

  /// Mounts what a spec names and hands a probe over it to `consumer`.
  pub fn with_probe<T>(&self, spec: &XrayRoots, consumer: impl FnOnce(&XrayProbe) -> T) -> TauriResult<T> {
    let mut vfs: MutexGuard<XrayVfs> = self
      .vfs
      .lock()
      .map_err(|error| format!("Failed to search assets - the mounted roots is unavailable: {error}"))?;

    let steps: Vec<XrayProbeStep> = spec
      .to_probe_plan()
      .map_err(|error| format!("Failed to plan the asset roots: {error}"))?
      .mount_into(&mut vfs)
      .map_err(|error| format!("Failed to mount the asset roots: {error}"))?;

    Ok(consumer(&vfs.probe().with_steps(steps)))
  }
}
