use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use xrf_vfs::{XrayProbe, XrayProbePlan, XrayProbeStep, XrayRoots, XrayVfs};

use crate::core::types::TauriResult;

/// Every mounted source the application holds, searched through per-request probes.
///
/// One VFS for the process rather than one per roots, because mounting is indexed eagerly and idempotent per planned
/// path: a viewer stepping through fifty models under one root pays for one index instead of fifty. Callers never
/// receive the VFS itself, only a probe over the steps their spec asked for, so an unscoped lookup cannot silently span
/// two unrelated roots.
#[derive(Clone)]
pub struct AssetMountState {
  vfs: Arc<RwLock<XrayVfs>>,
}

impl AssetMountState {
  pub fn new() -> Self {
    Self {
      vfs: Arc::new(RwLock::new(XrayVfs::new())),
    }
  }

  /// Mounts what a spec names and hands a probe over it to `consumer`.
  pub fn with_probe<T>(&self, spec: &XrayRoots, consumer: impl FnOnce(&XrayProbe) -> T) -> TauriResult<T> {
    let plan: XrayProbePlan = spec
      .to_probe_plan()
      .map_err(|error| format!("Failed to plan the asset roots: {error}"))?;
    let steps: Vec<XrayProbeStep> = self.mount(&plan)?;

    Ok(consumer(&self.read()?.probe().with_steps(steps)))
  }

  /// The steps of a plan, found among the mounts already held where it can be, and mounted where it cannot.
  fn mount(&self, plan: &XrayProbePlan) -> TauriResult<Vec<XrayProbeStep>> {
    let found: Option<Vec<XrayProbeStep>> = plan.find_mounted(&*self.read()?);

    if let Some(steps) = found {
      return Ok(steps);
    }

    let mut vfs: RwLockWriteGuard<XrayVfs> = self
      .vfs
      .write()
      .map_err(|error| format!("Failed to mount assets - the mounted roots are unavailable: {error}"))?;

    plan
      .mount_into(&mut vfs)
      .map_err(|error| format!("Failed to mount the asset roots: {error}"))
  }

  fn read(&self) -> TauriResult<RwLockReadGuard<'_, XrayVfs>> {
    self
      .vfs
      .read()
      .map_err(|error| format!("Failed to search assets - the mounted roots are unavailable: {error}"))
  }
}
