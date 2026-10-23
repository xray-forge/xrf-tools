use std::sync::{Arc, Mutex, MutexGuard, PoisonError, RwLock};

use tauri::ipc::Channel;
use xrf_error::XrfResult;
use xrf_vfs::{
  XrayAssetType, XrayCachePolicy, XrayCacheStats, XrayChangeSet, XrayRevision, XrayRoots, XraySearch, XrayTransaction,
  XrayVfs, XrayVfsHost,
};

use crate::core::assets::asset_change_order::AssetChangeOrder;
use crate::core::assets::asset_change_ticket::AssetChangeTicket;
use crate::core::assets::asset_fan_out::AssetFanOut;
use crate::core::assets::{AssetChangeEvent, AssetChangeSubscriber};
use crate::core::session::{SessionChange, SessionFollower};
use crate::core::types::TauriResult;

/// A published change's place in the order it is heard in, with the snapshot it published.
type AssetPublished = (AssetChangeTicket, Arc<XrayVfs>);

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
  /// The order every change is heard in, which is the order the changes were published in. No lock is held while
  /// sessions move or anyone hears, since a follow reading on the worker pool may take a job that commits or announces.
  order: Arc<AssetChangeOrder>,
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
      order: Arc::default(),
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

  /// The same for a person opening the roots: planned from disk again, every source that failed to open tried again,
  /// so a volume locked once, by a running game or a scan, opens once it is free, and every retained loose file the
  /// roots reach looked at on disk, so one edited behind the app is read afresh and every followed session moves as
  /// after a commit.
  pub fn search_fresh(&self, spec: &XrayRoots) -> TauriResult<XraySearch> {
    let opened: XraySearch = self.host.search_fresh(spec).map_err(|error| error.to_string())?;
    let (changes, published) = self.publish(|host| host.apply_changed_on_disk(&opened))?;

    let search: XraySearch = match published {
      None => opened,
      Some(published) => {
        log::info!(
          "Read {} retained file(s) changed on disk again, at {:?}",
          changes.get_changes().len(),
          changes.get_revision()
        );

        self.fan_out(published, changes.get_revision());

        // The plan is settled by now, so this is the snapshot the change published, or a newer one.
        self.search(spec)?
      }
    };
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

  /// A search over every writable loose mount, for a writer naming the files it commits on disk rather than by logical
  /// path: a file it does not hold is one no snapshot reads.
  pub fn search_writable(&self) -> XraySearch {
    self.host.search_writable()
  }

  /// Writes a transaction's files, all or none, and publishes the snapshot they leave, which every search taken after
  /// reads; then moves every followed session to it, tells every subscriber and then every listening page. Answers
  /// what changed.
  ///
  /// The sessions move before this returns, so a caller reading its own session afterwards sees what it committed, and
  /// the subscribers hear of it before this returns unless an earlier change is still being heard, which then tells of
  /// this one after it; a session deriving its value off the commit is reported `Deriving`, and what it derived comes
  /// later through [`Self::announce_sessions`].
  pub fn commit(&self, transaction: &XrayTransaction) -> TauriResult<XrayChangeSet> {
    let (changes, published) = self.publish(|host| host.commit(transaction))?;

    // todo: Record the transaction in an undo history with what it replaced and the editor it came from, capped by total
    //  bytes, oldest dropped; undo and redo then commit through here as transactions of their own.

    if let Some(published) = published {
      self.fan_out(published, changes.get_revision());
    }

    Ok(changes)
  }

  /// Runs a change through the host and, where it moved the project revision, takes the change's place in the order
  /// events are heard in, with the snapshot it published.
  fn publish(
    &self,
    change: impl FnOnce(&XrayVfsHost) -> XrfResult<XrayChangeSet>,
  ) -> TauriResult<(XrayChangeSet, Option<AssetPublished>)> {
    let ((changes, vfs), ticket) = self
      .order
      .issue(|| {
        let changes: XrayChangeSet = change(&self.host)?;
        // Every change takes the order's lock, so nothing newer than this one has published yet.
        let vfs: Arc<XrayVfs> = self.host.get_vfs();
        // Nothing written is nothing to follow: the project revision did not move.
        let is_changed: bool = !changes.get_changes().is_empty();

        XrfResult::Ok(((changes, vfs), is_changed))
      })
      .map_err(|error| error.to_string())?;

    Ok((changes, ticket.map(|ticket| (ticket, vfs))))
  }

  /// Moves every followed session to the snapshot a change at `revision` published, then has everyone hear of it in its
  /// turn.
  fn fan_out(&self, (ticket, vfs): AssetPublished, revision: XrayRevision) {
    let fan_out: AssetFanOut = AssetFanOut::new(vfs);
    let sessions: Vec<SessionChange> = self.move_sessions(&fan_out);

    self.deliver(ticket, AssetChangeEvent { revision, sessions }, fan_out);
  }

  /// Tells every subscriber and then every listening page what moved `sessions` after their commit's fan-out, a
  /// derivation that landed off the commit, as a change at the project revision held now, then starts what their
  /// follows put off for `fan_out`.
  pub(crate) fn announce_sessions(&self, sessions: Vec<SessionChange>, fan_out: AssetFanOut) {
    let ticket: AssetChangeTicket = self.order.issue_next();
    let event: AssetChangeEvent = AssetChangeEvent {
      revision: self.get_vfs().get_revision(),
      sessions,
    };

    self.deliver(ticket, event, fan_out);
  }

  /// Moves `follower` to the newest snapshot at once, telling everyone what that did: for an open that published
  /// after a change it did not read, which that change's fan-out found nothing to move for.
  pub(crate) fn catch_up(&self, follower: &dyn SessionFollower) {
    let fan_out: AssetFanOut = AssetFanOut::new(self.get_vfs());

    match follower.follow_in(self, &fan_out) {
      Ok(changes) if !changes.is_empty() => self.announce_sessions(changes, fan_out),
      Ok(_) => {
        for start in fan_out.into_starts() {
          start();
        }
      }
      Err(error) => log::error!("A session could not catch up with a commit: {error}"),
    }
  }

  /// Tells every subscriber what `event` did, then every listening page, then runs what the sessions' follows put off
  /// for it, all in `ticket`'s turn.
  fn deliver(&self, ticket: AssetChangeTicket, event: AssetChangeEvent, fan_out: AssetFanOut) {
    let state: AssetMountState = self.clone();

    ticket.deliver(move || {
      // Copied out, so a subscriber reading through this state never runs under the subscribers' lock.
      let subscribers: Vec<Arc<dyn AssetChangeSubscriber>> =
        state.subscribers.read().unwrap_or_else(PoisonError::into_inner).clone();

      for subscriber in &subscribers {
        subscriber.take_change(&state, &event);
      }

      state.announce(event);

      for start in fan_out.into_starts() {
        start();
      }
    });
  }

  /// Follows `session` through every commit from now on.
  pub(crate) fn add_follower(&self, session: impl SessionFollower + 'static) {
    self
      .followers
      .write()
      .unwrap_or_else(PoisonError::into_inner)
      .push(Arc::new(session));
  }

  /// Tells `subscriber` what every commit from now on did, in the backend.
  pub(crate) fn add_subscriber(&self, subscriber: impl AssetChangeSubscriber + 'static) {
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

  /// Moves each followed session to the snapshot `fan_out` carries, answering what that did to the ones it reached. A
  /// session whose lock is poisoned is passed over and said so, so one broken session leaves the others moving.
  fn move_sessions(&self, fan_out: &AssetFanOut) -> Vec<SessionChange> {
    // Copied out, so a derivation reading through this state never runs under the followers' lock.
    let followers: Vec<Arc<dyn SessionFollower>> =
      self.followers.read().unwrap_or_else(PoisonError::into_inner).clone();

    followers
      .iter()
      .flat_map(|follower| {
        follower
          .follow_in(self, fan_out)
          .inspect_err(|error| log::error!("A session could not follow a commit: {error}"))
          .unwrap_or_default()
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
