use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LockResult, Mutex, MutexGuard, RwLock};

use xrf_error::{XrfError, XrfResult};

use crate::{
  XrayChange, XrayChangeSet, XrayRoots, XraySearch, XraySearchPlan, XraySearchStep, XrayTransaction, XrayVfs,
};

/// The one owner of the current VFS snapshot, through which every mount goes: what an analysis host is to its analyses.
///
/// One snapshot for the process rather than one per request, because mounting is indexed eagerly and idempotent per
/// planned path: a viewer stepping through fifty models under one root pays for one index instead of fifty. Callers
/// never receive the snapshot to look up through, only a search over the steps their roots asked for, so an unscoped
/// lookup cannot silently span two unrelated roots.
///
/// Mounting is copy-on-write: a search reads an immutable snapshot and holds no lock, and a mount forks the newest
/// snapshot, mounts into the fork and publishes it whole. A search never waits for a mount and a mount never waits for
/// a search, however long either takes; what a search cannot see is a mount published after its snapshot was taken.
/// The forks share what the first snapshot retains, so a mount costs no parsed asset.
#[derive(Debug)]
pub struct XrayVfsHost {
  /// The newest snapshot, locked only for as long as it is taken or a fork replaces it.
  current: RwLock<Arc<XrayVfs>>,
  /// Taken by a mount for its whole run, so two mounting at once do not each publish a fork missing the other's.
  mounting: Mutex<()>,
  /// Plans per roots, planned from disk once and again only when a person opens them afresh.
  plans: Mutex<HashMap<XrayRoots, Arc<XraySearchPlan>>>,
  is_poison_reported: AtomicBool,
}

impl XrayVfsHost {
  /// A host whose first snapshot is `vfs`, which decides the cache policy and read trace every later one shares.
  pub fn new(vfs: XrayVfs) -> Self {
    Self {
      current: RwLock::new(Arc::new(vfs)),
      mounting: Mutex::new(()),
      plans: Mutex::new(HashMap::new()),
      is_poison_reported: AtomicBool::new(false),
    }
  }

  /// The newest snapshot, which stays what it is for as long as its holder keeps it.
  pub fn get_vfs(&self) -> Arc<XrayVfs> {
    self.recover(self.current.read()).clone()
  }

  /// A search over what `roots` names, mounted where it is not yet, a source that failed to open before staying
  /// settled: every ordinary read goes through here, and trying a broken volume on each would cost every read.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be planned or a plan cannot be applied.
  pub fn search(&self, roots: &XrayRoots) -> XrfResult<XraySearch> {
    let plan: Arc<XraySearchPlan> = self.plan(roots)?;
    let (vfs, steps): (Arc<XrayVfs>, Vec<XraySearchStep>) = self.mount(&plan, false)?;

    Ok(XraySearch::new(vfs).with_steps(steps))
  }

  /// The same for a person opening the roots: planned from disk again, and every source that failed to open tried
  /// again, so a volume locked once, by a running game or a scan, opens once it is free.
  ///
  /// # Errors
  ///
  /// As [`Self::search`].
  pub fn search_fresh(&self, roots: &XrayRoots) -> XrfResult<XraySearch> {
    self.plans().remove(roots);

    let plan: Arc<XraySearchPlan> = self.plan(roots)?;
    let (vfs, steps): (Arc<XrayVfs>, Vec<XraySearchStep>) = self.mount(&plan, true)?;

    Ok(XraySearch::new(vfs).with_steps(steps))
  }

  /// Commits a transaction against the newest snapshot and publishes the snapshot it leaves, answering what changed for
  /// the caller to fan out. Serialized with every mount, so neither publishes a snapshot missing the other's work.
  ///
  /// # Errors
  ///
  /// As [`XrayVfs::commit`], with nothing on disk or in the host changed.
  pub fn commit(&self, transaction: &XrayTransaction) -> XrfResult<XrayChangeSet> {
    self.change(|current| current.commit(transaction))
  }

  /// Applies changes already on disk to the newest snapshot and publishes the snapshot they leave, as
  /// [`XrayVfs::apply`] does.
  ///
  /// # Errors
  ///
  /// As [`XrayVfs::apply`], with nothing in the host changed.
  pub fn apply(&self, changes: Vec<XrayChange>) -> XrfResult<XrayChangeSet> {
    self.change(|current| current.apply(changes))
  }

  /// Runs a change against the newest snapshot under the mount lock and publishes the snapshot it answers.
  fn change(&self, change: impl FnOnce(&XrayVfs) -> XrfResult<(XrayVfs, XrayChangeSet)>) -> XrfResult<XrayChangeSet> {
    let _mounting: MutexGuard<()> = self.recover(self.mounting.lock());
    let (next, changes): (XrayVfs, XrayChangeSet) = change(&self.get_vfs())?;

    *self.recover(self.current.write()) = Arc::new(next);

    Ok(changes)
  }

  /// The search plan of some roots, planned from disk only the first time it is asked for.
  fn plan(&self, roots: &XrayRoots) -> XrfResult<Arc<XraySearchPlan>> {
    if let Some(plan) = self.plans().get(roots) {
      return Ok(plan.clone());
    }

    let plan: Arc<XraySearchPlan> = Arc::new(
      roots
        .to_search_plan()
        .map_err(|error| XrfError::new_asset_error(format!("Failed to plan the asset roots: {error}")))?,
    );

    Ok(self.plans().entry(roots.clone()).or_insert(plan).clone())
  }

  /// The snapshot a plan is searched in and its steps: the newest one where it settles every source the plan names,
  /// and a fork mounting the rest where it does not, or where `is_retrying` asks for the failures to be tried again.
  fn mount(&self, plan: &XraySearchPlan, is_retrying: bool) -> XrfResult<(Arc<XrayVfs>, Vec<XraySearchStep>)> {
    if let Some(found) = Self::find_settled(&self.get_vfs(), plan, is_retrying) {
      return Ok(found);
    }

    let _mounting: MutexGuard<()> = self.recover(self.mounting.lock());
    // Another mount may have settled the plan while this one waited for its turn.
    let current: Arc<XrayVfs> = self.get_vfs();

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
      .map_err(|error| XrfError::new_asset_error(format!("Failed to mount the asset roots: {error}")))?;
    let published: Arc<XrayVfs> = Arc::new(fork);

    *self.recover(self.current.write()) = published.clone();

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

  fn plans(&self) -> MutexGuard<'_, HashMap<XrayRoots, Arc<XraySearchPlan>>> {
    self.recover(self.plans.lock())
  }

  /// Takes a guard a panicking holder poisoned, which is sound because a guarded value is only ever replaced whole.
  fn recover<G>(&self, result: LockResult<G>) -> G {
    result.unwrap_or_else(|error| {
      if !self.is_poison_reported.swap(true, Ordering::Relaxed) {
        log::warn!("Recovering the VFS host after a panic while it was held");
      }

      error.into_inner()
    })
  }
}

impl Default for XrayVfsHost {
  fn default() -> Self {
    Self::new(XrayVfs::new())
  }
}
