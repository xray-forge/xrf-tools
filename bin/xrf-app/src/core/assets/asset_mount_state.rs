use std::sync::{Arc, Mutex, MutexGuard, PoisonError, RwLock};

use tauri::ipc::Channel;
use xrf_vfs::{
  XrayAssetType, XrayCachePolicy, XrayCacheStats, XrayChangeSet, XrayRoots, XraySearch, XrayTransaction, XrayVfs,
  XrayVfsHost,
};

use crate::core::assets::{AssetChangeEvent, AssetChangeSubscriber};
use crate::core::session::{SessionChange, SessionFollower};
use crate::core::types::TauriResult;

/// Work a session's follow puts off until the fan-out under way has told everyone.
type AssetFanOutStart = Box<dyn FnOnce() + Send>;

/// The application's VFS host, as the state every asset read takes a search from, and what moves every open session
/// along when a commit changes the files.
#[derive(Clone)]
pub struct AssetMountState {
  host: Arc<XrayVfsHost>,
  /// The sessions a commit moves to their next snapshot, each followed from its plugin's setup on.
  followers: Arc<RwLock<Vec<Arc<dyn SessionFollower>>>>,
  /// What follows commits in the backend, from its plugin's setup on.
  subscribers: Arc<RwLock<Vec<Arc<dyn AssetChangeSubscriber>>>>,
  /// The pages told what each commit did, until one stops listening.
  listeners: Arc<Mutex<Vec<Channel<AssetChangeEvent>>>>,
  /// What a session's follow put off until everyone has heard the change under way: a derivation started off the
  /// commit, whose own announcement must come after it.
  after_fan_out: Arc<Mutex<Vec<AssetFanOutStart>>>,
}

impl AssetMountState {
  /// What the process may retain of parsed configs, models and texture descriptors, across every snapshot and every
  /// reader, before it drops what no reader is holding: twice what one session held steady on Anomaly (2026-10-23:
  /// Zaton, ten models opened twice and Jupiter left 30.3 MiB of source bytes in 1,433 entries), a ceiling, not a
  /// working budget.
  const CACHE_BUDGET: u64 = 64 * 1024 * 1024;

  pub fn new() -> Self {
    Self {
      host: Arc::new(XrayVfsHost::new(XrayVfs::new().with_cache_policy(Self::cache_policy()))),
      followers: Arc::new(RwLock::new(Vec::new())),
      subscribers: Arc::new(RwLock::new(Vec::new())),
      listeners: Arc::new(Mutex::new(Vec::new())),
      after_fan_out: Arc::new(Mutex::new(Vec::new())),
    }
  }

  /// Configs, models and texture descriptors: what the viewers and editors reopen constantly. Textures stay out, since
  /// they are large and the renderer keeps its own.
  fn cache_policy() -> XrayCachePolicy {
    XrayCachePolicy::configs()
      .with(XrayAssetType::Ogf)
      .with(XrayAssetType::Thm)
      .with_budget(Self::CACHE_BUDGET)
  }

  /// The snapshot every search taken now reads, for a session checking what it was read from against it.
  pub fn get_vfs(&self) -> Arc<XrayVfs> {
    self.host.get_vfs()
  }

  /// A search over what a spec names, mounted where it is not yet, a source that failed to open before staying
  /// settled: every asset read goes through here, and trying a broken volume on each would cost every read.
  pub fn search(&self, spec: &XrayRoots) -> TauriResult<XraySearch> {
    self.host.search(spec).map_err(|error| error.to_string())
  }

  /// The same for a person opening the roots: planned from disk again, and every source that failed to open tried
  /// again, so a volume locked once, by a running game or a scan, opens once it is free.
  pub fn search_fresh(&self, spec: &XrayRoots) -> TauriResult<XraySearch> {
    let search: XraySearch = self.host.search_fresh(spec).map_err(|error| error.to_string())?;
    let stats: XrayCacheStats = search.get_vfs().get_cache_stats();

    // Once per open a person makes, which is what the ceiling above is judged by.
    log::info!(
      "Asset cache holds {} entries, {:.1} MiB ({} hits, {} misses, {} refused)",
      stats.entries,
      stats.bytes as f64 / (1024.0 * 1024.0),
      stats.hits,
      stats.misses,
      stats.refused
    );

    Ok(search)
  }

  /// Writes a transaction's files, all or none, and publishes the snapshot they leave, which every search taken after
  /// reads; then moves every followed session to it, tells every subscriber and then every listening page. Answers
  /// what changed.
  ///
  /// The sessions move and the subscribers hear of it before this returns, so a caller reading its own session
  /// afterwards sees what it committed; a session deriving its value off the commit is reported `Deriving`, and what it
  /// derived comes later through [`Self::announce_sessions`].
  pub fn commit(&self, transaction: &XrayTransaction) -> TauriResult<XrayChangeSet> {
    let changes: XrayChangeSet = self.host.commit(transaction).map_err(|error| error.to_string())?;

    // Nothing written is nothing to follow: the project revision did not move.
    if !changes.get_changes().is_empty() {
      self.fan_out(AssetChangeEvent {
        revision: changes.get_revision(),
        sessions: self.move_sessions(),
      });
    }

    Ok(changes)
  }

  /// Tells every subscriber and then every listening page what moved `sessions` after their commit's fan-out, a
  /// derivation that landed off the commit, as a commit's change at the project revision held now.
  pub(crate) fn announce_sessions(&self, sessions: Vec<SessionChange>) {
    self.fan_out(AssetChangeEvent {
      revision: self.get_vfs().get_revision(),
      sessions,
    });
  }

  /// Runs `start` once the fan-out under way has told every subscriber and page, for a session's follow that starts
  /// work announcing a later change of its own, so that change is never heard before this one.
  pub(crate) fn after_fan_out(&self, start: impl FnOnce() + Send + 'static) {
    self
      .after_fan_out
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .push(Box::new(start));
  }

  /// Tells every subscriber what `event` did, then every listening page, then runs what the sessions' follows put off.
  fn fan_out(&self, event: AssetChangeEvent) {
    // Copied out, so a subscriber reading through this state never runs under the subscribers' lock.
    let subscribers: Vec<Arc<dyn AssetChangeSubscriber>> =
      self.subscribers.read().unwrap_or_else(PoisonError::into_inner).clone();

    for subscriber in &subscribers {
      subscriber.take_change(self, &event);
    }

    self.announce(event);

    let started: Vec<AssetFanOutStart> =
      std::mem::take(&mut *self.after_fan_out.lock().unwrap_or_else(PoisonError::into_inner));

    for start in started {
      start();
    }
  }

  /// Follows `session` through every commit from now on.
  pub(crate) fn follow(&self, session: impl SessionFollower + 'static) {
    self
      .followers
      .write()
      .unwrap_or_else(PoisonError::into_inner)
      .push(Arc::new(session));
  }

  /// Tells `subscriber` what every commit from now on did, in the backend.
  pub(crate) fn subscribe(&self, subscriber: impl AssetChangeSubscriber + 'static) {
    self
      .subscribers
      .write()
      .unwrap_or_else(PoisonError::into_inner)
      .push(Arc::new(subscriber));
  }

  /// Tells `channel` what every commit from now on did, until the page behind it goes away.
  pub(crate) fn listen(&self, channel: Channel<AssetChangeEvent>) {
    self.lock_listeners().push(channel);
  }

  /// Moves each followed session to the snapshot now current, answering what that did to the ones it reached. A
  /// session whose lock is poisoned is passed over and said so, so one broken session leaves the others moving.
  fn move_sessions(&self) -> Vec<SessionChange> {
    // Copied out, so a derivation reading through this state never runs under the followers' lock.
    let followers: Vec<Arc<dyn SessionFollower>> =
      self.followers.read().unwrap_or_else(PoisonError::into_inner).clone();

    followers
      .iter()
      .filter_map(|follower| {
        follower
          .follow(self)
          .inspect_err(|error| log::error!("A session could not follow a commit: {error}"))
          .ok()
          .flatten()
      })
      .collect()
  }

  /// Sends `event` to every listening page, dropping the ones whose page is gone.
  fn announce(&self, event: AssetChangeEvent) {
    self
      .lock_listeners()
      .retain(|channel| match channel.send(event.clone()) {
        Ok(()) => true,
        Err(error) => {
          log::debug!("Dropped a change listener: {error}");

          false
        }
      });
  }

  /// A poisoned lock still holds live channels; refusing to send because one push once panicked would leave every page
  /// showing what it showed before.
  fn lock_listeners(&self) -> MutexGuard<'_, Vec<Channel<AssetChangeEvent>>> {
    self.listeners.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
