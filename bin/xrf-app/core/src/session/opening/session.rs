use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};
use std::time::Instant;

use rayon::ThreadPool;
use xrf_vfs::{XrayInputRecord, XrayRevision, XrayVfs};

use crate::TauriResult;
use crate::assets::{AssetFanOut, AssetMountState};
use crate::session::opening::SessionState;
use crate::session::opening::{SessionDerivation, SessionDerive};
use crate::session::{SessionChange, SessionDerived, SessionId, SessionParts, SessionRootChange, SessionSnapshot};

/// Publishes session snapshots atomically, retaining the last success while a replacement is pending: one handle's slot
/// in [`crate::session::Sessions`], which a domain opens through.
pub(crate) struct Session<T> {
  name: &'static str,
  state: Arc<Mutex<SessionState<T>>>,
  derivation: Option<SessionDerivation<T>>,
  /// The parts a view drawn from the session is built from, whose going stale renews it.
  view_parts: &'static [&'static str],
}

impl<T> Clone for Session<T> {
  fn clone(&self) -> Self {
    Self {
      name: self.name,
      state: Arc::clone(&self.state),
      derivation: self.derivation.clone(),
      view_parts: self.view_parts,
    }
  }
}

impl<T> Session<T> {
  /// An empty slot named `name` in what it says, deriving nothing and drawing no view.
  pub(crate) fn new(name: &'static str) -> Self {
    Self {
      name,
      state: Self::empty(),
      derivation: None,
      view_parts: &[],
    }
  }

  /// An empty session with this one's name, derivation and view parts, for a registry keeping one a handle.
  pub(crate) fn fresh(&self) -> Self {
    Self {
      name: self.name,
      state: Self::empty(),
      derivation: self.derivation.clone(),
      view_parts: self.view_parts,
    }
  }

  fn empty() -> Arc<Mutex<SessionState<T>>> {
    Arc::new(Mutex::new(SessionState {
      opening: None,
      opened: None,
      deriving: None,
      stale: Weak::new(),
    }))
  }

  /// The session with the derivation its plugin opens with, run inside the commit when it moved what the value was read
  /// from. Without one, such a value is kept and reported stale.
  pub(crate) fn with_derivation(
    mut self,
    derive: impl Fn(&SessionSnapshot<T>, &AssetMountState) -> TauriResult<SessionDerived<T>> + Send + Sync + 'static,
  ) -> Self {
    self.derivation = Some(SessionDerivation::Inline(Arc::new(derive)));

    self
  }

  /// The session with a derivation run on `pool` instead, which the commit does not wait for: it reports the session
  /// `Deriving`, and the derived snapshot is published and announced as `Derived`, or its failure as `Failed`, once it
  /// lands, unless an open, a close or another change published meanwhile.
  pub(crate) fn with_deferred_derivation(
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

  /// The session with the parts a view drawn from it is built from: a change that lets one of them go, the value kept,
  /// renews the view (`SessionChange::is_view_renewed`), as a derived value always does.
  pub(crate) fn with_view_parts(mut self, parts: &'static [&'static str]) -> Self {
    self.view_parts = parts;

    self
  }

  /// Reserves publication before doing work, without invalidating the committed value's readers.
  pub(crate) fn begin_open(&self, id: SessionId) {
    self.lock().opening = Some(id);
  }

  /// Gives up the opening `id` reserved, keeping whatever the slot committed before it.
  pub(crate) fn give_up_open(&self, id: SessionId) {
    let mut state: MutexGuard<SessionState<T>> = self.lock();

    if state.opening == Some(id) {
      state.opening = None;
    }
  }

  /// Publishes only the newest opening; parsing and resource acquisition happen before this call.
  #[cfg(test)]
  pub(crate) fn commit_open(&self, id: SessionId, value: T) -> TauriResult<Arc<SessionSnapshot<T>>> {
    self.commit_open_with(
      id,
      value,
      XrayRevision::MOUNTED,
      XrayInputRecord::new(),
      SessionParts::new(),
    )
  }

  /// Publishes only the newest opening as [`Self::commit_open`] does, at `revision`, the one the search it read was
  /// taken at, keeping the inputs the value was derived from and the parts the opening read on its way.
  pub(crate) fn commit_open_with(
    &self,
    id: SessionId,
    value: T,
    revision: XrayRevision,
    inputs: XrayInputRecord,
    parts: SessionParts,
  ) -> TauriResult<Arc<SessionSnapshot<T>>> {
    let opened: Arc<SessionSnapshot<T>> = Arc::new(SessionSnapshot {
      revision,
      inputs,
      parts,
      ..SessionSnapshot::new(id, value)
    });

    let mut state: MutexGuard<SessionState<T>> = self.lock();

    if state.opening != Some(id) {
      return Err(self.superseded());
    }

    let previous: Option<Arc<SessionSnapshot<T>>> = state.opened.replace(Arc::clone(&opened));

    state.opening = None;

    drop(state);
    drop(previous);

    Ok(opened)
  }

  /// What the session is called in what it says.
  pub(crate) fn get_name(&self) -> &'static str {
    self.name
  }

  /// What a commit says of an opening it no longer publishes: closed while it opened, or replaced in a slot held alone.
  fn superseded(&self) -> String {
    format!("The {} opening was closed or replaced while it opened", self.name)
  }

  /// Restores the committed value without copying its content or exposing the session lock.
  pub(crate) fn get(&self) -> Option<Arc<SessionSnapshot<T>>> {
    self.lock().opened.clone()
  }

  /// Takes the exact value requested, never whichever value happens to be open now.
  pub(crate) fn require(&self, id: SessionId) -> TauriResult<Arc<SessionSnapshot<T>>> {
    self
      .get()
      .filter(|opened| opened.session_id == id)
      .ok_or_else(|| format!("The {} session has changed or is closed", self.name))
  }

  /// Closes only the caller's committed or pending openings. A delayed teardown cannot close its successor.
  #[cfg(test)]
  pub(crate) fn close(&self, ids: &[SessionId]) {
    drop(self.detach(ids));
  }

  /// Opens `value` as `id` at `revision` with the inputs and parts it was read with, for a test following a slot
  /// through commits without a registry or a scope.
  #[cfg(test)]
  pub(crate) fn commit_fixture(
    &self,
    id: SessionId,
    value: T,
    revision: XrayRevision,
    inputs: XrayInputRecord,
    parts: SessionParts,
  ) -> TauriResult<Arc<SessionSnapshot<T>>> {
    self.begin_open(id);
    self.commit_open_with(id, value, revision, inputs, parts)
  }

  /// Detaches ownership so a caller can dispose a large value on its execution pool.
  pub(crate) fn detach(&self, ids: &[SessionId]) -> Option<Arc<SessionSnapshot<T>>> {
    let mut state: MutexGuard<SessionState<T>> = self.lock();

    if state.opening.is_some_and(|id| ids.contains(&id)) {
      state.opening = None;
    }

    if state
      .opened
      .as_ref()
      .is_some_and(|opened| ids.contains(&opened.session_id))
    {
      state.opened.take()
    } else {
      None
    }
  }

  /// Forgets the derivation of `current`, where it is the one under way.
  fn end_deriving(state: &mut SessionState<T>, current: &Arc<SessionSnapshot<T>>) {
    if state
      .deriving
      .as_ref()
      .is_some_and(|deriving| Arc::ptr_eq(deriving, current))
    {
      state.deriving = None;
    }
  }

  /// The slot, recovered from a poisoned lock: every change replaces a field whole, so a panic between two leaves a
  /// state that merely lags, which a session refusing every read afterwards would turn into a document nobody can use.
  fn lock(&self) -> MutexGuard<'_, SessionState<T>> {
    self.state.lock().unwrap_or_else(PoisonError::into_inner)
  }
}

impl<T: Send + Sync + 'static> Session<T> {
  /// Moves the committed snapshot to the project revision `assets` holds now, starting at once what the follow puts
  /// off, for a test that announces nothing.
  #[cfg(test)]
  pub(crate) fn follow(&self, assets: &AssetMountState) -> Option<SessionChange> {
    let fan_out: AssetFanOut = AssetFanOut::new(assets.get_vfs());
    let change: Option<SessionChange> = self.follow_in(assets, &fan_out);

    for start in fan_out.into_starts() {
      start();
    }

    change
  }

  /// Answers `opened`, just published, or, where a change published since the search it read was taken, what the
  /// session holds once it caught up with it, everyone told: no fan-out of that change found the opening to move.
  pub(crate) fn catch_up(&self, opened: Arc<SessionSnapshot<T>>, assets: &AssetMountState) -> Arc<SessionSnapshot<T>> {
    if assets.get_vfs().get_revision() <= opened.revision {
      return opened;
    }

    assets.catch_up(self);

    self.require(opened.session_id).unwrap_or(opened)
  }

  /// Moves the committed snapshot to the one `fan_out` carries, the change already on disk.
  ///
  /// Where nothing the value was read from moved, the next snapshot shares the value and carries every part whose
  /// inputs are current; where something did, the plugin's derivation reads the value again and the next snapshot
  /// starts without parts, and without a derivation the snapshot stays as it is, reported stale once. A deferred
  /// derivation is reported `Deriving` and started once `fan_out` has been heard, one reading the same snapshot again
  /// already left to land. A derivation's failure is reported, never dropped, and leaves the snapshot as it was, to be
  /// tried again by the next change. Nothing that moved since is lost to a race: an open, a close or another change
  /// published meanwhile is followed from instead.
  pub(crate) fn follow_in(&self, assets: &AssetMountState, fan_out: &AssetFanOut) -> Option<SessionChange> {
    let vfs: &Arc<XrayVfs> = fan_out.get_vfs();
    let revision: XrayRevision = vfs.get_revision();

    loop {
      let current: Arc<SessionSnapshot<T>> = self.get()?;

      if current.revision >= revision || self.is_reported_stale(&current) {
        return None;
      }

      let session_id: SessionId = current.session_id;
      let (next, root, stale_parts) = if current.inputs.is_current_in(vfs) {
        let (parts, stale_parts) = current.parts.to_current(vfs);
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
            return Some(self.defer(&current, derive, pool, assets, fan_out));
          }
          None => {
            if !self.mark_stale(&current) {
              continue;
            }

            return Some(self.to_change(
              session_id,
              current.revision,
              SessionRootChange::Stale,
              current.parts.list_built(),
            ));
          }
        };

        match derive(&current, assets) {
          Ok(derived) => (
            Self::to_derived(&current, revision, derived),
            SessionRootChange::Derived,
            current.parts.list_built(),
          ),
          Err(reason) => return Some(self.to_failed(&current, reason)),
        }
      };

      if self.publish_next(&current, next) {
        return Some(self.to_change(session_id, revision, root, stale_parts));
      }
    }
  }

  /// Whether `current` is the snapshot reported stale already.
  fn is_reported_stale(&self, current: &Arc<SessionSnapshot<T>>) -> bool {
    std::ptr::eq(self.lock().stale.as_ptr(), Arc::as_ptr(current))
  }

  /// Remembers `current` as reported stale while it is still the committed snapshot, answering whether it was.
  fn mark_stale(&self, current: &Arc<SessionSnapshot<T>>) -> bool {
    let mut state: MutexGuard<SessionState<T>> = self.lock();

    if !state.opened.as_ref().is_some_and(|opened| Arc::ptr_eq(opened, current)) {
      return false;
    }

    state.stale = Arc::downgrade(current);

    true
  }

  /// Starts reading `current` again on `pool` once `fan_out` has been heard, unless a derivation of it is under way
  /// already, answering what the change reports of the session meanwhile. The start is guarded: dropped unrun, as a
  /// fan-out given up on its way drops it, it lets the derivation go, so the next change starts one again.
  fn defer(
    &self,
    current: &Arc<SessionSnapshot<T>>,
    derive: &Arc<SessionDerive<T>>,
    pool: &Arc<ThreadPool>,
    assets: &AssetMountState,
    fan_out: &AssetFanOut,
  ) -> SessionChange {
    let mut state: MutexGuard<SessionState<T>> = self.lock();
    let is_under_way: bool = state
      .deriving
      .as_ref()
      .is_some_and(|deriving| Arc::ptr_eq(deriving, current));

    if !is_under_way {
      state.deriving = Some(Arc::clone(current));
    }

    drop(state);

    if !is_under_way {
      let mut guard: SessionDerivingGuard<T> = SessionDerivingGuard {
        session: self.clone(),
        current: Arc::clone(current),
        is_started: false,
      };
      let (derive, pool, landing) = (Arc::clone(derive), Arc::clone(pool), AssetMountState::clone(assets));

      // Started only once this change is heard, so the derived one never overtakes it.
      fan_out.after(move || {
        guard.is_started = true;

        let (session, current) = (guard.session.clone(), Arc::clone(&guard.current));

        pool.spawn(move || session.land(&current, derive.as_ref(), &landing));
      });
    }

    self.to_change(
      current.session_id,
      current.revision,
      SessionRootChange::Deriving,
      Vec::new(),
    )
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

        let change: SessionChange = self.to_change(
          current.session_id,
          revision,
          SessionRootChange::Derived,
          current.parts.list_built(),
        );

        (Some(Self::to_derived(current, revision, derived)), change)
      }
      Err(reason) => (None, self.to_failed(current, reason)),
    };

    if self.settle(current, next) {
      assets.announce_sessions(vec![change], AssetFanOut::new(assets.get_vfs()));
    } else {
      log::info!(
        "A derivation of the {} session gave way to an open, a close or a change published while it read",
        self.name
      );
    }

    if assets.get_vfs().get_revision() > revision {
      assets.catch_up(self);
    }
  }

  /// Ends the derivation of `current`, publishing `next` in its place while it is still the committed snapshot;
  /// answers whether it was.
  fn settle(&self, current: &Arc<SessionSnapshot<T>>, next: Option<SessionSnapshot<T>>) -> bool {
    let next: Option<Arc<SessionSnapshot<T>>> = next.map(Arc::new);
    let mut state: MutexGuard<SessionState<T>> = self.lock();

    Self::end_deriving(&mut state, current);

    let is_current: bool = state.opened.as_ref().is_some_and(|opened| Arc::ptr_eq(opened, current));
    let previous: Option<Arc<SessionSnapshot<T>>> = match (is_current, next) {
      (true, Some(next)) => state.opened.replace(next),
      _ => None,
    };

    drop(state);
    drop(previous);

    is_current
  }

  /// Publishes `next` only while `current` is still the committed snapshot, answering whether it did.
  fn publish_next(&self, current: &Arc<SessionSnapshot<T>>, next: SessionSnapshot<T>) -> bool {
    let next: Arc<SessionSnapshot<T>> = Arc::new(next);
    let mut state: MutexGuard<SessionState<T>> = self.lock();

    if !state.opened.as_ref().is_some_and(|opened| Arc::ptr_eq(opened, current)) {
      return false;
    }

    let previous: Option<Arc<SessionSnapshot<T>>> = state.opened.replace(next);

    drop(state);
    drop(previous);

    true
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

    self.to_change(
      current.session_id,
      current.revision,
      SessionRootChange::Failed { reason },
      Vec::new(),
    )
  }

  /// What a change reports of the session, saying whether it renews a view drawn from it.
  fn to_change(
    &self,
    session_id: SessionId,
    revision: XrayRevision,
    root: SessionRootChange,
    stale_parts: Vec<&'static str>,
  ) -> SessionChange {
    let is_view_renewed: bool = match root {
      SessionRootChange::Derived => true,
      SessionRootChange::Kept => stale_parts.iter().any(|part| self.view_parts.contains(part)),
      SessionRootChange::Deriving | SessionRootChange::Stale | SessionRootChange::Failed { .. } => false,
    };

    SessionChange {
      session_id,
      revision,
      root,
      stale_parts,
      is_view_renewed,
    }
  }
}

/// A deferred derivation put off until its change is heard, which lets the session's derivation go where it is dropped
/// without having started: otherwise the session would answer `Deriving` to every later change and never derive again.
struct SessionDerivingGuard<T> {
  session: Session<T>,
  current: Arc<SessionSnapshot<T>>,
  is_started: bool,
}

impl<T> Drop for SessionDerivingGuard<T> {
  fn drop(&mut self) {
    if !self.is_started {
      Session::end_deriving(&mut self.session.lock(), &self.current);
    }
  }
}
