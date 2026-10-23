use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use rayon::ThreadPool;
use xrf_vfs::{XrayInputRecord, XrayRevision, XrayVfs};

use crate::core::assets::AssetMountState;
use crate::core::session::{
  SessionChange, SessionDerived, SessionId, SessionParts, SessionRootChange, SessionSnapshot,
};
use crate::core::types::TauriResult;

/// How a session's plugin derives its value again from the snapshot it replaces, reading through `assets` as its open
/// did.
type SessionDerive<T> = dyn Fn(&SessionSnapshot<T>, &AssetMountState) -> TauriResult<SessionDerived<T>> + Send + Sync;

/// Where a session's derivation runs once a commit moved what its value was read from.
enum SessionDerivation<T> {
  /// Inside the commit, on the committing thread, which a reader of the session after the commit relies on.
  Inline(Arc<SessionDerive<T>>),
  /// On `pool`, the commit reporting the session `Deriving` at once and the derived snapshot announced through the asset
  /// state's fan-out once it lands: for a derivation too slow to hold a save for.
  Deferred {
    derive: Arc<SessionDerive<T>>,
    pool: Arc<ThreadPool>,
  },
}

impl<T> Clone for SessionDerivation<T> {
  fn clone(&self) -> Self {
    match self {
      Self::Inline(derive) => Self::Inline(Arc::clone(derive)),
      Self::Deferred { derive, pool } => Self::Deferred {
        derive: Arc::clone(derive),
        pool: Arc::clone(pool),
      },
    }
  }
}

/// Publishes session snapshots atomically, retaining the last success while a replacement is pending.
pub(crate) struct Session<T> {
  name: &'static str,
  state: Arc<Mutex<SessionState<T>>>,
  derivation: Option<SessionDerivation<T>>,
}

impl<T> Clone for Session<T> {
  fn clone(&self) -> Self {
    Self {
      name: self.name,
      state: Arc::clone(&self.state),
      derivation: self.derivation.clone(),
    }
  }
}

struct SessionState<T> {
  opening: Option<SessionId>,
  opened: Option<Arc<SessionSnapshot<T>>>,
  /// The snapshot a deferred derivation is reading again, so a commit landing meanwhile starts no second one.
  deriving: Option<Arc<SessionSnapshot<T>>>,
}

impl<T> Session<T> {
  pub fn new(name: &'static str) -> Self {
    Self {
      name,
      state: Arc::new(Mutex::new(SessionState {
        opening: None,
        opened: None,
        deriving: None,
      })),
      derivation: None,
    }
  }

  /// The session with the derivation its plugin opens with, run inside the commit when it moved what the value was read
  /// from. Without one, such a value is kept and reported stale.
  pub fn with_derivation(
    mut self,
    derive: impl Fn(&SessionSnapshot<T>, &AssetMountState) -> TauriResult<SessionDerived<T>> + Send + Sync + 'static,
  ) -> Self {
    self.derivation = Some(SessionDerivation::Inline(Arc::new(derive)));

    self
  }

  /// The session with a derivation run on `pool` instead, which the commit does not wait for: it reports the session
  /// `Deriving`, and the derived snapshot is published and announced as `Derived`, or its failure as `Failed`, once it
  /// lands, unless an open, a close or another change published meanwhile.
  pub fn with_deferred_derivation(
    mut self,
    pool: Arc<ThreadPool>,
    derive: impl Fn(&SessionSnapshot<T>, &AssetMountState) -> TauriResult<SessionDerived<T>> + Send + Sync + 'static,
  ) -> Self {
    self.derivation = Some(SessionDerivation::Deferred {
      derive: Arc::new(derive),
      pool,
    });

    self
  }

  /// Reserves publication before doing work, without invalidating the committed value's readers.
  pub fn begin_open(&self, id: SessionId) -> TauriResult<()> {
    self.lock()?.opening = Some(id);

    Ok(())
  }

  /// Reserves a reload only while the value it reloads is still committed.
  pub fn begin_reload(&self, id: SessionId, previous: SessionId) -> TauriResult<Arc<SessionSnapshot<T>>> {
    let mut state: MutexGuard<SessionState<T>> = self.lock()?;

    let opened: Arc<SessionSnapshot<T>> = state
      .opened
      .as_ref()
      .filter(|opened| opened.session_id == previous)
      .cloned()
      .ok_or_else(|| format!("The {} session has changed or is closed", self.name))?;

    state.opening = Some(id);

    Ok(opened)
  }

  /// Whether this opening is still the one that would be published, for work that is worth abandoning when it is not.
  ///
  /// # Errors
  ///
  /// Returns an error when a newer open or a close has taken the opening, or when the lock is poisoned.
  pub fn require_opening(&self, id: SessionId) -> TauriResult<()> {
    if self.lock()?.opening == Some(id) {
      return Ok(());
    }

    Err(self.superseded())
  }

  /// Publishes only the newest opening; parsing and resource acquisition happen before this call.
  pub fn commit_open(&self, id: SessionId, value: T) -> TauriResult<Arc<SessionSnapshot<T>>> {
    self.commit_open_with(id, value, XrayInputRecord::new(), SessionParts::new())
  }

  /// Publishes only the newest opening as [`Self::commit_open`] does, keeping the inputs the value was derived from and
  /// the parts the opening read on its way.
  pub fn commit_open_with(
    &self,
    id: SessionId,
    value: T,
    inputs: XrayInputRecord,
    parts: SessionParts,
  ) -> TauriResult<Arc<SessionSnapshot<T>>> {
    let opened: Arc<SessionSnapshot<T>> = Arc::new(SessionSnapshot {
      inputs,
      parts,
      ..SessionSnapshot::new(id, value)
    });

    let mut state: MutexGuard<SessionState<T>> = self.lock()?;

    if state.opening != Some(id) {
      return Err(self.superseded());
    }

    let previous: Option<Arc<SessionSnapshot<T>>> = state.opened.replace(Arc::clone(&opened));

    state.opening = None;

    drop(state);
    drop(previous);

    Ok(opened)
  }

  /// What both the early check and the commit say, so a caller cannot tell them apart and need not.
  fn superseded(&self) -> String {
    format!("The {} opening was superseded by another open or close", self.name)
  }

  /// Restores the committed value without copying its content or exposing the session lock.
  pub fn get(&self) -> TauriResult<Option<Arc<SessionSnapshot<T>>>> {
    Ok(self.lock()?.opened.clone())
  }

  /// Takes the exact value requested, never whichever value happens to be open now.
  pub fn require(&self, id: SessionId) -> TauriResult<Arc<SessionSnapshot<T>>> {
    self
      .get()?
      .filter(|opened| opened.session_id == id)
      .ok_or_else(|| format!("The {} session has changed or is closed", self.name))
  }

  /// Closes only the caller's committed or pending openings. A delayed teardown cannot close its successor.
  pub fn close(&self, ids: &[SessionId]) -> TauriResult<()> {
    drop(self.detach(ids)?);

    Ok(())
  }

  /// Detaches ownership so a caller can dispose a large value on its execution pool.
  pub fn detach(&self, ids: &[SessionId]) -> TauriResult<Option<Arc<SessionSnapshot<T>>>> {
    let mut state: MutexGuard<SessionState<T>> = self.lock()?;

    if state.opening.is_some_and(|id| ids.contains(&id)) {
      state.opening = None;
    }

    let previous: Option<Arc<SessionSnapshot<T>>> = if state
      .opened
      .as_ref()
      .is_some_and(|opened| ids.contains(&opened.session_id))
    {
      state.opened.take()
    } else {
      None
    };

    drop(state);

    Ok(previous)
  }

  fn lock(&self) -> TauriResult<MutexGuard<'_, SessionState<T>>> {
    self
      .state
      .lock()
      .map_err(|error| format!("The {} session is unavailable: {error}", self.name))
  }
}

impl<T: Send + Sync + 'static> Session<T> {
  /// Moves the committed snapshot to the project revision `assets` holds now, the change already on disk.
  ///
  /// Where nothing the value was read from moved, the next snapshot shares the value and carries every part whose
  /// inputs are current; where something did, the plugin's derivation reads the value again and the next snapshot
  /// starts without parts, and without a derivation the snapshot stays as it is, reported stale. A deferred derivation
  /// is reported `Deriving` and started once the asset state's fan-out under way has told everyone, one reading the
  /// same snapshot again already left to land. A derivation's
  /// failure is reported, never dropped, and leaves the snapshot as it was. Nothing that moved since is lost to a race:
  /// an open, a close or another change published meanwhile is followed from instead.
  ///
  /// # Errors
  ///
  /// Returns an error when the session's lock is poisoned.
  pub fn follow(&self, assets: &AssetMountState) -> TauriResult<Option<SessionChange>> {
    let vfs: Arc<XrayVfs> = assets.get_vfs();
    let revision: XrayRevision = vfs.get_revision();

    loop {
      let Some(current) = self.get()? else {
        return Ok(None);
      };

      if current.revision >= revision {
        return Ok(None);
      }

      let session_id: SessionId = current.session_id;
      let (next, root, stale_parts) = if current.inputs.is_current_in(&vfs) {
        let (parts, stale_parts) = current.parts.retain_current(&vfs);
        let next: SessionSnapshot<T> = SessionSnapshot {
          session_id,
          revision,
          value: Arc::clone(&current.value),
          inputs: current.inputs.clone(),
          parts,
        };

        (next, SessionRootChange::Kept, stale_parts)
      } else {
        let derive: &Arc<SessionDerive<T>> = match &self.derivation {
          Some(SessionDerivation::Inline(derive)) => derive,
          Some(SessionDerivation::Deferred { derive, pool }) => {
            return self.defer(&current, derive, pool, assets).map(Some);
          }
          None => {
            return Ok(Some(SessionChange {
              session_id,
              revision: current.revision,
              root: SessionRootChange::Stale,
              stale_parts: current.parts.list_built(),
            }));
          }
        };

        match derive(&current, assets) {
          Ok(derived) => (
            Self::to_derived(&current, revision, derived),
            SessionRootChange::Derived,
            current.parts.list_built(),
          ),
          Err(reason) => return Ok(Some(self.to_failed(&current, reason))),
        }
      };

      if self.publish_next(&current, next)? {
        return Ok(Some(SessionChange {
          session_id,
          revision,
          root,
          stale_parts,
        }));
      }
    }
  }

  /// Starts reading `current` again on `pool` once the fan-out under way told everyone, unless a derivation of it is
  /// under way already, answering what the commit reports of the session meanwhile.
  fn defer(
    &self,
    current: &Arc<SessionSnapshot<T>>,
    derive: &Arc<SessionDerive<T>>,
    pool: &Arc<ThreadPool>,
    assets: &AssetMountState,
  ) -> TauriResult<SessionChange> {
    let mut state: MutexGuard<SessionState<T>> = self.lock()?;
    let is_under_way: bool = state
      .deriving
      .as_ref()
      .is_some_and(|deriving| Arc::ptr_eq(deriving, current));

    if !is_under_way {
      state.deriving = Some(Arc::clone(current));
    }

    drop(state);

    if !is_under_way {
      let (session, current, derive, pool, landing) = (
        self.clone(),
        Arc::clone(current),
        Arc::clone(derive),
        Arc::clone(pool),
        AssetMountState::clone(assets),
      );

      // Started only once this commit's own change is heard, so the derived one never overtakes it.
      assets.after_fan_out(move || pool.spawn(move || session.land(&current, derive.as_ref(), &landing)));
    }

    Ok(SessionChange {
      session_id: current.session_id,
      revision: current.revision,
      root: SessionRootChange::Deriving,
      stale_parts: Vec::new(),
    })
  }

  /// Reads `current` again as a deferred derivation, then publishes what it read while `current` is still the committed
  /// snapshot and announces it, or its failure, through `assets` as a commit's change; last, follows a commit that
  /// landed while it read, which left the session to it.
  fn land(&self, current: &Arc<SessionSnapshot<T>>, derive: &SessionDerive<T>, assets: &AssetMountState) {
    let started: Instant = Instant::now();
    let revision: XrayRevision = assets.get_vfs().get_revision();
    // A panic on the pool would take the process down with it, so it is reported as the derivation's failure.
    let derived: TauriResult<SessionDerived<T>> = panic::catch_unwind(AssertUnwindSafe(|| derive(current, assets)))
      .unwrap_or_else(|_| Err(String::from("the derivation panicked")));
    let (next, change) = match derived {
      Ok(derived) => {
        log::info!(
          "The {} session was derived again in {} ms",
          self.name,
          started.elapsed().as_millis()
        );

        let change: SessionChange = SessionChange {
          session_id: current.session_id,
          revision,
          root: SessionRootChange::Derived,
          stale_parts: current.parts.list_built(),
        };

        (Some(Self::to_derived(current, revision, derived)), change)
      }
      Err(reason) => (None, self.to_failed(current, reason)),
    };

    match self.settle(current, next) {
      Ok(true) => assets.announce_sessions(vec![change]),
      Ok(false) => log::info!(
        "A derivation of the {} session gave way to an open, a close or a change published while it read",
        self.name
      ),
      Err(error) => log::error!("{error}"),
    }

    if assets.get_vfs().get_revision() > revision {
      match self.follow(assets) {
        Ok(Some(change)) => assets.announce_sessions(vec![change]),
        Ok(None) => {}
        Err(error) => log::error!("The {} session could not follow a commit: {error}", self.name),
      }
    }
  }

  /// Ends the derivation of `current`, publishing `next` in its place while it is still the committed snapshot;
  /// answers whether it was.
  fn settle(&self, current: &Arc<SessionSnapshot<T>>, next: Option<SessionSnapshot<T>>) -> TauriResult<bool> {
    let next: Option<Arc<SessionSnapshot<T>>> = next.map(Arc::new);
    let mut state: MutexGuard<SessionState<T>> = self.lock()?;

    if state
      .deriving
      .as_ref()
      .is_some_and(|deriving| Arc::ptr_eq(deriving, current))
    {
      state.deriving = None;
    }

    let is_current: bool = state.opened.as_ref().is_some_and(|opened| Arc::ptr_eq(opened, current));
    let previous: Option<Arc<SessionSnapshot<T>>> = match (is_current, next) {
      (true, Some(next)) => state.opened.replace(next),
      _ => None,
    };

    drop(state);
    drop(previous);

    Ok(is_current)
  }

  /// Publishes `next` only while `current` is still the committed snapshot, answering whether it did.
  fn publish_next(&self, current: &Arc<SessionSnapshot<T>>, next: SessionSnapshot<T>) -> TauriResult<bool> {
    let next: Arc<SessionSnapshot<T>> = Arc::new(next);
    let mut state: MutexGuard<SessionState<T>> = self.lock()?;

    if !state.opened.as_ref().is_some_and(|opened| Arc::ptr_eq(opened, current)) {
      return Ok(false);
    }

    let previous: Option<Arc<SessionSnapshot<T>>> = state.opened.replace(next);

    drop(state);
    drop(previous);

    Ok(true)
  }

  /// The snapshot a derivation of `current` read at `revision` publishes: its identity, the derived value, and no part
  /// but those the derivation read on its way.
  fn to_derived(
    current: &SessionSnapshot<T>,
    revision: XrayRevision,
    derived: SessionDerived<T>,
  ) -> SessionSnapshot<T> {
    SessionSnapshot {
      session_id: current.session_id,
      revision,
      value: Arc::new(derived.value),
      inputs: derived.inputs,
      parts: derived.parts,
    }
  }

  /// What a failed derivation of `current` reports: the session stays at the revision it had.
  fn to_failed(&self, current: &SessionSnapshot<T>, reason: String) -> SessionChange {
    log::warn!("The {} session could not be derived again: {reason}", self.name);

    SessionChange {
      session_id: current.session_id,
      revision: current.revision,
      root: SessionRootChange::Failed { reason },
      stale_parts: Vec::new(),
    }
  }
}
