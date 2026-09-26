use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LockResult, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

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
  /// Plans per roots, never refreshed, which is no staler than the mounts they name.
  plans: Arc<Mutex<HashMap<XrayRoots, Arc<XrayProbePlan>>>>,
  is_poison_reported: Arc<AtomicBool>,
}

impl AssetMountState {
  pub fn new() -> Self {
    Self {
      vfs: Arc::new(RwLock::new(XrayVfs::new())),
      plans: Arc::new(Mutex::new(HashMap::new())),
      is_poison_reported: Arc::new(AtomicBool::new(false)),
    }
  }

  /// Mounts what a spec names and hands a probe over it to `consumer`.
  pub fn with_probe<T>(&self, spec: &XrayRoots, consumer: impl FnOnce(&XrayProbe) -> T) -> TauriResult<T> {
    let plan: Arc<XrayProbePlan> = self.plan(spec)?;
    let steps: Vec<XrayProbeStep> = self.mount(&plan)?;

    Ok(consumer(&self.read().probe().with_steps(steps)))
  }

  /// The probe plan of a spec, planned from disk only the first time it is asked for.
  fn plan(&self, spec: &XrayRoots) -> TauriResult<Arc<XrayProbePlan>> {
    if let Some(plan) = self.plans().get(spec) {
      return Ok(plan.clone());
    }

    let plan: Arc<XrayProbePlan> = Arc::new(
      spec
        .to_probe_plan()
        .map_err(|error| format!("Failed to plan the asset roots: {error}"))?,
    );

    Ok(self.plans().entry(spec.clone()).or_insert(plan).clone())
  }

  /// The steps of a plan, found among the mounts already held where it can be, and mounted where it cannot.
  fn mount(&self, plan: &XrayProbePlan) -> TauriResult<Vec<XrayProbeStep>> {
    let found: Option<Vec<XrayProbeStep>> = plan.find_mounted(&self.read());

    if let Some(steps) = found {
      return Ok(steps);
    }

    let mut vfs: RwLockWriteGuard<XrayVfs> = self.recover(self.vfs.write());

    plan
      .mount_into(&mut vfs)
      .map_err(|error| format!("Failed to mount the asset roots: {error}"))
  }

  fn read(&self) -> RwLockReadGuard<'_, XrayVfs> {
    self.recover(self.vfs.read())
  }

  fn plans(&self) -> MutexGuard<'_, HashMap<XrayRoots, Arc<XrayProbePlan>>> {
    self.recover(self.plans.lock())
  }

  /// Takes a guard a panicking holder poisoned, which is sound because mounting and planning only ever append.
  fn recover<G>(&self, result: LockResult<G>) -> G {
    result.unwrap_or_else(|error| {
      if !self.is_poison_reported.swap(true, Ordering::Relaxed) {
        log::warn!("Recovering the mounted asset roots after a panic while they were held");
      }

      error.into_inner()
    })
  }
}
