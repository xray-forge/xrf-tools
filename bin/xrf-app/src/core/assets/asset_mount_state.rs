use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LockResult, Mutex, MutexGuard, RwLock};

use xrf_vfs::{XrayRoots, XraySearch, XraySearchPlan, XraySearchStep, XrayVfs};

use crate::core::types::TauriResult;

/// Every mounted source the application holds, read through per-request searches.
///
/// One VFS for the process rather than one per roots, because mounting is indexed eagerly and idempotent per planned
/// path: a viewer stepping through fifty models under one root pays for one index instead of fifty. Callers never
/// receive the VFS itself, only a search over the steps their spec asked for, so an unscoped lookup cannot silently
/// span two unrelated roots.
///
/// Mounting is copy-on-write: a search reads an immutable VFS snapshot and holds no lock, and a mount forks the newest
/// snapshot, mounts into the fork and publishes it whole. A search never waits for a mount and a mount never waits for
/// a search, however long either takes; what a search cannot see is a mount published after its snapshot was taken.
#[derive(Clone)]
pub struct AssetMountState {
  /// The newest VFS snapshot, locked only for as long as it is taken or a fork replaces it.
  vfs: Arc<RwLock<Arc<XrayVfs>>>,
  /// Taken by a mount for its whole run, so two mounting at once do not each publish a fork missing the other's.
  mounting: Arc<Mutex<()>>,
  /// Plans per roots, planned from disk once and again only when an open asks for them afresh.
  plans: Arc<Mutex<HashMap<XrayRoots, Arc<XraySearchPlan>>>>,
  is_poison_reported: Arc<AtomicBool>,
}

impl AssetMountState {
  pub fn new() -> Self {
    Self {
      vfs: Arc::new(RwLock::new(Arc::new(XrayVfs::new()))),
      mounting: Arc::new(Mutex::new(())),
      plans: Arc::new(Mutex::new(HashMap::new())),
      is_poison_reported: Arc::new(AtomicBool::new(false)),
    }
  }

  /// A search over what a spec names, mounted where it is not yet, a source that failed to open before staying
  /// settled: every asset read goes through here, and trying a broken volume on each would cost every read.
  pub fn search(&self, spec: &XrayRoots) -> TauriResult<XraySearch> {
    let plan: Arc<XraySearchPlan> = self.plan(spec)?;
    let (vfs, steps): (Arc<XrayVfs>, Vec<XraySearchStep>) = self.mount(&plan, false)?;

    Ok(XraySearch::new(vfs).with_steps(steps))
  }

  /// The same for a person opening the roots: planned from disk again, and every source that failed to open tried
  /// again, so a volume locked once, by a running game or a scan, opens once it is free.
  pub fn search_fresh(&self, spec: &XrayRoots) -> TauriResult<XraySearch> {
    self.plans().remove(spec);

    let plan: Arc<XraySearchPlan> = self.plan(spec)?;
    let (vfs, steps): (Arc<XrayVfs>, Vec<XraySearchStep>) = self.mount(&plan, true)?;

    Ok(XraySearch::new(vfs).with_steps(steps))
  }

  /// Hands [`Self::search`]'s search to `consumer`.
  pub fn with_search<T>(&self, spec: &XrayRoots, consumer: impl FnOnce(&XraySearch) -> T) -> TauriResult<T> {
    Ok(consumer(&self.search(spec)?))
  }

  /// Hands [`Self::search_fresh`]'s search to `consumer`.
  pub fn with_fresh_search<T>(&self, spec: &XrayRoots, consumer: impl FnOnce(&XraySearch) -> T) -> TauriResult<T> {
    Ok(consumer(&self.search_fresh(spec)?))
  }

  /// The search plan of a spec, planned from disk only the first time it is asked for.
  fn plan(&self, spec: &XrayRoots) -> TauriResult<Arc<XraySearchPlan>> {
    if let Some(plan) = self.plans().get(spec) {
      return Ok(plan.clone());
    }

    let plan: Arc<XraySearchPlan> = Arc::new(
      spec
        .to_search_plan()
        .map_err(|error| format!("Failed to plan the asset roots: {error}"))?,
    );

    Ok(self.plans().entry(spec.clone()).or_insert(plan).clone())
  }

  /// The snapshot a plan is searched in and its steps: the newest one where it settles every source the plan names,
  /// and a fork mounting the rest where it does not, or where `is_retrying` asks for the failures to be tried again.
  fn mount(&self, plan: &XraySearchPlan, is_retrying: bool) -> TauriResult<(Arc<XrayVfs>, Vec<XraySearchStep>)> {
    if let Some(found) = Self::find_settled(&self.snapshot(), plan, is_retrying) {
      return Ok(found);
    }

    let _mounting: MutexGuard<()> = self.recover(self.mounting.lock());
    // Another mount may have settled the plan while this one waited for its turn.
    let current: Arc<XrayVfs> = self.snapshot();

    if let Some(found) = Self::find_settled(&current, plan, is_retrying) {
      return Ok(found);
    }

    let mut fork: XrayVfs = current.fork();

    // Only this plan's failures: another's stay settled for every read of it.
    if is_retrying {
      plan.forget_skipped_in(&mut fork);
    }

    let steps: Vec<XraySearchStep> = plan
      .mount_into(&mut fork)
      .map_err(|error| format!("Failed to mount the asset roots: {error}"))?;
    let published: Arc<XrayVfs> = Arc::new(fork);

    *self.recover(self.vfs.write()) = published.clone();

    Ok((published, steps))
  }

  /// A snapshot's steps for a plan where every source it names is settled there, and, for a retry, none of them by a
  /// failure.
  fn find_settled(
    vfs: &Arc<XrayVfs>,
    plan: &XraySearchPlan,
    is_retrying: bool,
  ) -> Option<(Arc<XrayVfs>, Vec<XraySearchStep>)> {
    plan
      .find_mounted(vfs)
      .filter(|_| !is_retrying || !plan.has_failed_in(vfs))
      .map(|steps| (vfs.clone(), steps))
  }

  /// The newest snapshot, which stays what it is for as long as its holder keeps it.
  fn snapshot(&self) -> Arc<XrayVfs> {
    self.recover(self.vfs.read()).clone()
  }

  fn plans(&self) -> MutexGuard<'_, HashMap<XrayRoots, Arc<XraySearchPlan>>> {
    self.recover(self.plans.lock())
  }

  /// Takes a guard a panicking holder poisoned, which is sound because a guarded value is only ever replaced whole.
  fn recover<G>(&self, result: LockResult<G>) -> G {
    result.unwrap_or_else(|error| {
      if !self.is_poison_reported.swap(true, Ordering::Relaxed) {
        log::warn!("Recovering the mounted asset roots after a panic while they were held");
      }

      error.into_inner()
    })
  }
}
