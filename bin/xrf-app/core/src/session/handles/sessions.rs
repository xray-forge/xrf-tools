use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use rayon::ThreadPool;
use xrf_vfs::{XrayInputRecord, XrayRevision, XrayRoots, XraySearch};

use crate::TauriResult;
use crate::assets::{AssetFanOut, AssetMountState};
use crate::execution::ExecutionState;
use crate::session::handles::sessions_state::SessionsState;
use crate::session::{
  Session, SessionBindings, SessionChange, SessionDerived, SessionFollower, SessionId, SessionOpening, SessionParts,
  SessionRelease, SessionScope, SessionScopes, SessionSnapshot,
};

/// Every opening of one kind of document, each a handle its opener closes: the many a domain holds at once, one for each
/// tool and document. Each handle publishes its snapshots as one slot does, and is followed through commits with the
/// rest where the openings derive their value again. A clone answers for the same handles, and opens new ones with the
/// same derivation.
///
/// A handle opens only once its scope is bound, so whatever releases the scope closes it; what a closed handle held is
/// dropped on the scopes' pool.
pub struct Sessions<T> {
  state: Arc<Mutex<SessionsState<T>>>,
  bindings: SessionBindings,
}

impl<T> Clone for Sessions<T> {
  fn clone(&self) -> Self {
    Self {
      state: Arc::clone(&self.state),
      bindings: self.bindings.clone(),
    }
  }
}

impl<T: Send + Sync + 'static> Sessions<T> {
  /// The openings of one kind, named `name` in what they say, closed with whatever scope `scopes` releases.
  pub fn new(name: &'static str, scopes: &SessionScopes) -> Self {
    let sessions: Self = Self {
      state: Arc::new(Mutex::new(SessionsState::new(Session::new(name)))),
      bindings: scopes.get_bindings(),
    };

    scopes.register(sessions.clone());

    sessions
  }

  /// The openings read again by `derive` inside the commit that moved what one was read from, followed through every
  /// commit `assets` publishes from now on: a reader of a session after its commit sees what was derived.
  pub fn with_derivation(
    self,
    assets: &AssetMountState,
    derive: impl Fn(&SessionSnapshot<T>, &AssetMountState) -> TauriResult<SessionDerived<T>> + Send + Sync + 'static,
  ) -> Self {
    self.update_template(|template| template.with_derivation(derive));
    assets.add_follower(self.clone());

    self
  }

  /// The openings read again by `derive` on `pool`, which the commit does not wait for, followed through every commit
  /// `assets` publishes from now on: such a session is reported `Deriving`, and what was derived announced once it
  /// lands, unless an open, a close or another change published meanwhile.
  pub fn with_deferred_derivation(
    self,
    assets: &AssetMountState,
    pool: Arc<ThreadPool>,
    derive: impl Fn(&SessionSnapshot<T>, &AssetMountState) -> TauriResult<SessionDerived<T>> + Send + Sync + 'static,
  ) -> Self {
    self.update_template(|template| template.with_deferred_derivation(pool, derive));
    assets.add_follower(self.clone());

    self
  }

  /// The openings with the parts a view drawn from one is built from: a change that lets one of them go, the value kept,
  /// renews the view (`SessionChange::is_view_renewed`), as a derived value always does.
  pub fn with_view_parts(self, parts: &'static [&'static str]) -> Self {
    self.update_template(|template| template.with_view_parts(parts));

    self
  }

  /// Reserves a handle's publication before doing work, answering the opening that commits it, or abandons it as it
  /// drops; other handles, open or opening, are left as they are.
  ///
  /// The handle is reserved before its binding is asked for: a release unbinds before it closes, so one landing before
  /// the ask is seen here, and one landing after finds the handle to close.
  ///
  /// # Errors
  ///
  /// Returns an error when `id` is bound to no scope: a handle nothing releases would outlive its tool.
  pub fn begin_open(&self, id: SessionId) -> TauriResult<SessionOpening<T>> {
    let handle: Session<T> = {
      let mut state: MutexGuard<'_, SessionsState<T>> = self.lock();
      let fresh: Session<T> = state.template.fresh();

      state.handles.entry(id).or_insert(fresh).clone()
    };
    // Made at once, so whatever refuses the open below abandons the handle with it.
    let opening: SessionOpening<T> = SessionOpening::new(self.clone(), id);

    handle.begin_open(id);

    if !self.bindings.is_bound(id) {
      return Err(format!(
        "The {} opening is bound to no tool, so nothing would close it",
        handle.get_name()
      ));
    }

    Ok(opening)
  }

  /// Opens `id` with what `read` reads on the application's pool, a value no commit moves: the handle is reserved
  /// before the read, and given up where the read fails.
  ///
  /// # Errors
  ///
  /// Returns an error when the handle cannot be reserved, the read fails, or the handle was closed while it opened.
  pub async fn open_on_pool(
    &self,
    execution: &ExecutionState,
    id: SessionId,
    what: &str,
    read: impl FnOnce() -> TauriResult<T> + Send + 'static,
  ) -> TauriResult<Arc<SessionSnapshot<T>>> {
    let opening: SessionOpening<T> = self.begin_open(id)?;
    let value: T = execution.run_blocking(what, read).await??;

    opening.commit(value)
  }

  /// Opens `id` with what `read` reads, on the application's pool, through a fresh search of `roots`: the search a
  /// person's open takes, recording what it reads so a commit to any of it reads the value again. The handle is
  /// reserved before the read and given up where it fails; it is published at the search's revision and caught up with
  /// any change published since.
  ///
  /// # Errors
  ///
  /// Returns an error when the handle cannot be reserved, the roots cannot be searched, the read fails, or the handle
  /// was closed while it opened.
  pub async fn open_followed(
    &self,
    execution: &ExecutionState,
    assets: &AssetMountState,
    id: SessionId,
    roots: XrayRoots,
    what: &str,
    read: impl FnOnce(XraySearch) -> TauriResult<SessionDerived<T>> + Send + 'static,
  ) -> TauriResult<Arc<SessionSnapshot<T>>> {
    let opening: SessionOpening<T> = self.begin_open(id)?;
    let reader: AssetMountState = AssetMountState::clone(assets);
    let (read, search) = execution
      .run_blocking(what, move || {
        let search: XraySearch = reader.search_fresh(&roots)?;

        TauriResult::Ok((read(search.clone())?, search))
      })
      .await??;

    opening.commit_read(read, &search, assets)
  }

  /// Reserves `id` to open again what `previous` holds, while `previous` is still open, answering the opening with what
  /// `previous` holds: a reload is a new handle, which the opener closes `previous` for once it commits.
  ///
  /// # Errors
  ///
  /// Returns an error when `previous` holds nothing, or `id` cannot be reserved.
  pub fn begin_reload(
    &self,
    id: SessionId,
    previous: SessionId,
  ) -> TauriResult<(SessionOpening<T>, Arc<SessionSnapshot<T>>)> {
    let opened: Arc<SessionSnapshot<T>> = self.require(previous)?;

    Ok((self.begin_open(id)?, opened))
  }

  /// Binds `id` to a scope of its own and reserves it, for a test opening a handle no tool asked for.
  #[cfg(any(test, feature = "fixtures"))]
  pub fn begin_fixture_open(&self, id: SessionId) -> TauriResult<SessionOpening<T>> {
    self.bindings.bind(id, SessionScope::new("fixture", id.to_string()));

    self.begin_open(id)
  }

  /// Publishes a handle's opening at `revision`, with the inputs its value was read from and the parts it read on its
  /// way. A handle closed as it commits is not one a restore answers.
  pub(super) fn commit_open_with(
    &self,
    id: SessionId,
    value: T,
    revision: XrayRevision,
    inputs: XrayInputRecord,
    parts: SessionParts,
  ) -> TauriResult<Arc<SessionSnapshot<T>>> {
    let opened: Arc<SessionSnapshot<T>> = self
      .require_handle(id)?
      .commit_open_with(id, value, revision, inputs, parts)?;
    let mut state: MutexGuard<'_, SessionsState<T>> = self.lock();

    if state.handles.contains_key(&id) {
      state.sequence += 1;

      let sequence: u64 = state.sequence;

      state.committed.insert(id, sequence);
    }

    Ok(opened)
  }

  /// Lets go of a handle whose open was given up before it committed. One that committed before is the opener's to
  /// close: it keeps what it holds, and only the open given up is forgotten.
  pub(super) fn abandon(&self, id: SessionId) {
    let handle: Option<Session<T>> = {
      let mut state: MutexGuard<'_, SessionsState<T>> = self.lock();

      if state.committed.contains_key(&id) {
        let handle: Option<Session<T>> = state.handles.get(&id).cloned();

        drop(state);

        if let Some(handle) = handle {
          handle.give_up_open(id);
        }

        return;
      }

      state.handles.remove(&id)
    };

    if let Some(handle) = handle {
      drop(handle.detach(&[id]));
    }

    self.bindings.unbind(&[id]);
  }

  /// The snapshot a handle holds, never another's.
  ///
  /// # Errors
  ///
  /// Returns an error when the handle is closed or holds nothing yet.
  pub fn require(&self, id: SessionId) -> TauriResult<Arc<SessionSnapshot<T>>> {
    self.require_handle(id)?.require(id)
  }

  /// Answers what `read` reads from the snapshot `id` holds, on the application's pool, under the snapshot's identity
  /// and revision, so a page can tell an answer older than what it shows.
  ///
  /// # Errors
  ///
  /// Returns an error when the handle holds nothing, the read fails, or the blocking hop does.
  pub async fn read_on_pool<U: Send + Sync + 'static>(
    &self,
    execution: &ExecutionState,
    id: SessionId,
    what: &str,
    read: impl FnOnce(&SessionSnapshot<T>) -> TauriResult<U> + Send + 'static,
  ) -> TauriResult<SessionSnapshot<U>> {
    let current: Arc<SessionSnapshot<T>> = self.require(id)?;

    execution
      .run_blocking(what, move || Ok(current.answer(read(&current)?)))
      .await?
  }

  /// The newest committed snapshot among the handles `scope` holds, for a page attaching to what its tool holds; none
  /// where none of them holds one.
  pub fn restore(&self, scope: &SessionScope) -> Option<Arc<SessionSnapshot<T>>> {
    let ids: Vec<SessionId> = self.bindings.list(scope);
    let newest: Option<Session<T>> = {
      let state: MutexGuard<'_, SessionsState<T>> = self.lock();

      ids
        .iter()
        .filter_map(|id| Some((*state.committed.get(id)?, state.handles.get(id)?)))
        .max_by_key(|(sequence, _)| *sequence)
        .map(|(_, handle)| handle.clone())
    };

    newest.and_then(|handle| handle.get())
  }

  /// What [`Self::restore`] answers, described by `describe` under the snapshot's identity and revision.
  pub fn restore_as<U>(&self, scope: &SessionScope, describe: impl FnOnce(&T) -> U) -> Option<SessionSnapshot<U>> {
    self.restore(scope).map(|opened| opened.map(describe))
  }

  /// Every committed snapshot, for a check that refuses work over whatever any handle holds.
  pub fn list_opened(&self) -> Vec<Arc<SessionSnapshot<T>>> {
    let handles: Vec<Session<T>> = self.lock().handles.values().cloned().collect();

    handles.iter().filter_map(Session::get).collect()
  }

  /// Answers `opened`, just published, or what its handle holds once it caught up with a change published since the
  /// search it read was taken.
  pub(crate) fn catch_up(&self, opened: Arc<SessionSnapshot<T>>, assets: &AssetMountState) -> Arc<SessionSnapshot<T>> {
    match self.find_handle(opened.session_id) {
      Some(handle) => handle.catch_up(opened, assets),
      None => opened,
    }
  }

  /// Closes the caller's handles, committed or pending, and only those, dropping what they held on the scopes' pool.
  pub fn close(&self, ids: &[SessionId]) {
    let (name, handles): (&'static str, Vec<(SessionId, Session<T>)>) = {
      let mut state: MutexGuard<'_, SessionsState<T>> = self.lock();
      let handles: Vec<(SessionId, Session<T>)> = ids
        .iter()
        .filter_map(|id| {
          state.committed.remove(id);
          state.handles.remove(id).map(|handle| (*id, handle))
        })
        .collect();

      (state.template.get_name(), handles)
    };

    // Unbound whether or not a handle was reserved yet, so an id closed before its open began opens no more.
    self.bindings.unbind(ids);

    if handles.is_empty() {
      return;
    }

    log::info!("Closing {} {name} handle(s)", handles.len());

    // Each handle is closed as well as forgotten, so a derivation of it landing later publishes nothing.
    let detached: Vec<Arc<SessionSnapshot<T>>> = handles
      .into_iter()
      .filter_map(|(id, handle)| handle.detach(&[id]))
      .collect();

    if !detached.is_empty() {
      self.bindings.dispose(detached);
    }
  }

  /// Replaces what new handles start as, for every clone at once, so the clone a scope release holds opens nothing
  /// with a template the plugin has since replaced.
  fn update_template(&self, update: impl FnOnce(Session<T>) -> Session<T>) {
    let mut state: MutexGuard<'_, SessionsState<T>> = self.lock();
    let template: Session<T> = state.template.clone();

    state.template = update(template);
  }

  fn require_handle(&self, id: SessionId) -> TauriResult<Session<T>> {
    let state: MutexGuard<'_, SessionsState<T>> = self.lock();

    state
      .handles
      .get(&id)
      .cloned()
      .ok_or_else(|| format!("The {} session has changed or is closed", state.template.get_name()))
  }

  fn find_handle(&self, id: SessionId) -> Option<Session<T>> {
    self.lock().handles.get(&id).cloned()
  }

  /// The handles, whatever a panicking caller left: a handle is only ever added or removed whole.
  fn lock(&self) -> MutexGuard<'_, SessionsState<T>> {
    self.state.lock().unwrap_or_else(PoisonError::into_inner)
  }
}

impl<T: Send + Sync + 'static> SessionFollower for Sessions<T> {
  /// Follows every handle, each answering what moved it.
  fn follow_in(&self, assets: &AssetMountState, fan_out: &AssetFanOut) -> Vec<SessionChange> {
    // Copied out, so a derivation reading inside the follow never runs under the handles' lock.
    let handles: Vec<Session<T>> = self.lock().handles.values().cloned().collect();

    handles
      .iter()
      .filter_map(|handle| handle.follow_in(assets, fan_out))
      .collect()
  }
}

impl<T: Send + Sync + 'static> SessionRelease for Sessions<T> {
  fn release(&self, ids: &[SessionId]) {
    self.close(ids);
  }
}
