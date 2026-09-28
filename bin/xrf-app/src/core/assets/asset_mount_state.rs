use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LockResult, Mutex, MutexGuard, RwLock};

use xrf_vfs::{XrayProbe, XrayProbePlan, XrayProbeStep, XrayRoots, XrayVfs};

use crate::core::types::TauriResult;

/// Every mounted source the application holds, searched through per-request probes.
///
/// One VFS for the process rather than one per roots, because mounting is indexed eagerly and idempotent per planned
/// path: a viewer stepping through fifty models under one root pays for one index instead of fifty. Callers never
/// receive the VFS itself, only a probe over the steps their spec asked for, so an unscoped lookup cannot silently span
/// two unrelated roots.
///
/// Mounting is copy-on-write: a consumer is handed an immutable snapshot of the world and holds no lock while it runs,
/// and a mount forks the newest snapshot, mounts into the fork and publishes it whole. A consumer never waits for a
/// mount and a mount never waits for a consumer, however long either takes, and a consumer may ask for a probe of its
/// own; what it cannot see is a mount published after its snapshot was taken.
#[derive(Clone)]
pub struct AssetMountState {
  /// The newest world, locked only for as long as a snapshot is taken of it or a fork replaces it.
  world: Arc<RwLock<Arc<XrayVfs>>>,
  /// Taken by a mount for its whole run, so two mounting at once do not each publish a fork missing the other's.
  mounting: Arc<Mutex<()>>,
  /// Plans per roots, planned from disk once and again only when an open asks for them afresh.
  plans: Arc<Mutex<HashMap<XrayRoots, Arc<XrayProbePlan>>>>,
  is_poison_reported: Arc<AtomicBool>,
}

impl AssetMountState {
  pub fn new() -> Self {
    Self {
      world: Arc::new(RwLock::new(Arc::new(XrayVfs::new()))),
      mounting: Arc::new(Mutex::new(())),
      plans: Arc::new(Mutex::new(HashMap::new())),
      is_poison_reported: Arc::new(AtomicBool::new(false)),
    }
  }

  /// Mounts what a spec names and hands a probe over it to `consumer`, a source that failed to open before staying
  /// settled: every asset read goes through here, and trying a broken volume on each would cost every read.
  pub fn with_probe<T>(&self, spec: &XrayRoots, consumer: impl FnOnce(&XrayProbe) -> T) -> TauriResult<T> {
    let plan: Arc<XrayProbePlan> = self.plan(spec)?;
    let (world, steps): (Arc<XrayVfs>, Vec<XrayProbeStep>) = self.mount(&plan, false)?;

    Ok(consumer(&world.probe().with_steps(steps)))
  }

  /// The same for a person opening the roots: planned from disk again, and every source that failed to open tried
  /// again, so a volume locked once, by a running game or a scan, opens once it is free.
  pub fn with_fresh_probe<T>(&self, spec: &XrayRoots, consumer: impl FnOnce(&XrayProbe) -> T) -> TauriResult<T> {
    self.plans().remove(spec);

    let plan: Arc<XrayProbePlan> = self.plan(spec)?;
    let (world, steps): (Arc<XrayVfs>, Vec<XrayProbeStep>) = self.mount(&plan, true)?;

    Ok(consumer(&world.probe().with_steps(steps)))
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

  /// The world a plan is searched in and its steps: the newest snapshot where it settles every source the plan names,
  /// and a fork mounting the rest where it does not, or where `is_retrying` asks for the failures to be tried again.
  fn mount(&self, plan: &XrayProbePlan, is_retrying: bool) -> TauriResult<(Arc<XrayVfs>, Vec<XrayProbeStep>)> {
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

    if is_retrying {
      fork.forget_skipped_mounts();
    }

    let steps: Vec<XrayProbeStep> = plan
      .mount_into(&mut fork)
      .map_err(|error| format!("Failed to mount the asset roots: {error}"))?;
    let published: Arc<XrayVfs> = Arc::new(fork);

    *self.recover(self.world.write()) = published.clone();

    Ok((published, steps))
  }

  /// A world's steps for a plan where every source it names is settled there, and, for a retry, none of them by a
  /// failure.
  fn find_settled(
    world: &Arc<XrayVfs>,
    plan: &XrayProbePlan,
    is_retrying: bool,
  ) -> Option<(Arc<XrayVfs>, Vec<XrayProbeStep>)> {
    plan
      .find_mounted(world)
      .filter(|steps| !is_retrying || steps.iter().all(|step| step.get_skipped().is_empty()))
      .map(|steps| (world.clone(), steps))
  }

  /// The newest world, which stays what it is for as long as its holder keeps it.
  fn snapshot(&self) -> Arc<XrayVfs> {
    self.recover(self.world.read()).clone()
  }

  fn plans(&self) -> MutexGuard<'_, HashMap<XrayRoots, Arc<XrayProbePlan>>> {
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
