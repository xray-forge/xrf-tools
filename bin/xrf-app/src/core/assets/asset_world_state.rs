use std::sync::{Mutex, MutexGuard};

use xrf_vfs::{XrayProbe, XrayProbeStep, XrayVfs, XrayWorldSpec};

use crate::core::types::TauriResult;

/// Every mounted source the application holds, searched through per-request probes.
///
/// One VFS for the process rather than one per world, because mounting is indexed eagerly and idempotent per planned
/// path: a viewer stepping through fifty models under one root pays for one index instead of fifty. Callers never
/// receive the VFS itself, only a probe over the steps their spec asked for, so an unscoped lookup cannot silently span
/// two unrelated worlds.
///
/// Lives in `core/` because it belongs to no command domain: visuals resolves a model's textures through it, and the
/// surfaces that follow — an archive preview, a level view — mount the same world instead of indexing their own.
pub struct AssetWorldState {
  vfs: Mutex<XrayVfs>,
}

impl AssetWorldState {
  pub fn new() -> Self {
    Self {
      vfs: Mutex::new(XrayVfs::new()),
    }
  }

  /// Mounts what a spec names and hands a probe over it to `consumer`.
  ///
  /// Scoped to a closure because a probe borrows the VFS the lock protects: returning one would either leak the guard or
  /// outlive it. It also keeps mounting and searching in one critical section, so two commands opening the same root
  /// cannot both index it.
  ///
  /// `asset`, when given, is searched for beside itself first — its own X-Ray root, then the installation containing it —
  /// which is how the engine finds a texture shipped next to a model rather than in the shared tree.
  pub fn with_probe<T>(&self, spec: &XrayWorldSpec, consumer: impl FnOnce(&XrayProbe) -> T) -> TauriResult<T> {
    let mut vfs: MutexGuard<XrayVfs> = self
      .vfs
      .lock()
      .map_err(|error| format!("Failed to search assets - the mounted world is unavailable: {error}"))?;

    let steps: Vec<XrayProbeStep> = spec
      .to_probe_plan()
      .map_err(|error| format!("Failed to plan the asset world: {error}"))?
      .mount_into(&mut vfs)
      .map_err(|error| format!("Failed to mount the asset world: {error}"))?;

    Ok(consumer(&vfs.probe().with_steps(steps)))
  }
}
