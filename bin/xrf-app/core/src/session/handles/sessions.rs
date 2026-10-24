use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use rayon::ThreadPool;
use xrf_vfs::{XrayInputRecord, XrayRevision};

use crate::assets::{AssetFanOut, AssetMountState};
use crate::session::handles::sessions_state::SessionsState;
use crate::session::{
  Session, SessionChange, SessionDerived, SessionFollower, SessionId, SessionParts, SessionRelease, SessionScopes,
  SessionSnapshot,
};
use crate::types::TauriResult;

/// Every opening of one kind of document, each a handle its opener closes: the many a domain holds at once, one for each
/// tool and document. Each handle publishes its snapshots as one [`Session`] does, and is followed through commits with
/// the rest. A clone answers for the same handles, and opens new ones with the same derivation.
pub struct Sessions<T> {
  state: Arc<Mutex<SessionsState<T>>>,
}

impl<T> Clone for Sessions<T> {
  fn clone(&self) -> Self {
    Self {
      state: Arc::clone(&self.state),
    }
  }
}

impl<T: Send + Sync + 'static> Sessions<T> {
  /// The openings of one kind, named `name` in what they say, closed with whatever scope `scopes` releases.
  pub fn new(name: &'static str, scopes: &SessionScopes) -> Self {
    let sessions: Self = Self {
      state: Arc::new(Mutex::new(SessionsState::new(Session::new(name)))),
    };

    scopes.register(sessions.clone());

    sessions
  }

  /// The openings with the derivation their plugin opens with, run inside the commit; see
  /// [`Session::with_derivation`].
  pub fn with_derivation(
    self,
    derive: impl Fn(&SessionSnapshot<T>, &AssetMountState) -> TauriResult<SessionDerived<T>> + Send + Sync + 'static,
  ) -> Self {
    self.update_template(|template| template.with_derivation(derive));

    self
  }

  /// The openings with a derivation run on `pool`, which the commit does not wait for; see
  /// [`Session::with_deferred_derivation`].
  pub fn with_deferred_derivation(
    self,
    pool: Arc<ThreadPool>,
    derive: impl Fn(&SessionSnapshot<T>, &AssetMountState) -> TauriResult<SessionDerived<T>> + Send + Sync + 'static,
  ) -> Self {
    self.update_template(|template| template.with_deferred_derivation(pool, derive));

    self
  }

  /// The openings with the parts a view drawn from one is built from; see [`Session::with_view_parts`].
  pub fn with_view_parts(self, parts: &'static [&'static str]) -> Self {
    self.update_template(|template| template.with_view_parts(parts));

    self
  }

  /// Reserves a handle's publication before doing work; other handles, open or opening, are left as they are.
  pub fn begin_open(&self, id: SessionId) -> TauriResult<()> {
    let handle: Session<T> = {
      let mut state: MutexGuard<'_, SessionsState<T>> = self.lock();
      let fresh: Session<T> = state.template.fresh();

      state.handles.entry(id).or_insert(fresh).clone()
    };

    handle.begin_open(id)
  }

  /// Reserves `id` to open again what `previous` holds, while `previous` is still open, answering what it holds: a
  /// reload is a new handle, which the opener closes `previous` for once it commits.
  ///
  /// # Errors
  ///
  /// Returns an error when `previous` holds nothing, or `id` cannot be reserved.
  pub fn begin_reload(&self, id: SessionId, previous: SessionId) -> TauriResult<Arc<SessionSnapshot<T>>> {
    let opened: Arc<SessionSnapshot<T>> = self.require(previous)?;

    self.begin_open(id)?;

    Ok(opened)
  }

  /// Publishes a handle's opening; see [`Session::commit_open`].
  pub fn commit_open(&self, id: SessionId, value: T) -> TauriResult<Arc<SessionSnapshot<T>>> {
    self.commit_open_with(
      id,
      value,
      XrayRevision::MOUNTED,
      XrayInputRecord::new(),
      SessionParts::new(),
    )
  }

  /// Publishes a handle's opening at `revision`, with the inputs its value was read from and the parts it read on its
  /// way; see [`Session::commit_open_with`]. A handle closed as it commits is not one a restore answers.
  pub fn commit_open_with(
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

  /// The snapshot a handle holds, never another's.
  pub fn require(&self, id: SessionId) -> TauriResult<Arc<SessionSnapshot<T>>> {
    self.require_handle(id)?.require(id)
  }

  /// The newest committed snapshot among `ids` still open, for a page attaching to what its scope holds; none where
  /// none of them holds one.
  pub fn restore(&self, ids: &[SessionId]) -> TauriResult<Option<Arc<SessionSnapshot<T>>>> {
    let newest: Option<Session<T>> = {
      let state: MutexGuard<'_, SessionsState<T>> = self.lock();

      ids
        .iter()
        .filter_map(|id| Some((*state.committed.get(id)?, state.handles.get(id)?)))
        .max_by_key(|(sequence, _)| *sequence)
        .map(|(_, handle)| handle.clone())
    };

    match newest {
      Some(handle) => handle.get(),
      None => Ok(None),
    }
  }

  /// Every committed snapshot, for a check that refuses work over whatever any handle holds.
  pub fn list_opened(&self) -> TauriResult<Vec<Arc<SessionSnapshot<T>>>> {
    let handles: Vec<Session<T>> = self.lock().handles.values().cloned().collect();
    let mut opened: Vec<Arc<SessionSnapshot<T>>> = Vec::new();

    for handle in handles {
      opened.extend(handle.get()?);
    }

    Ok(opened)
  }

  /// Answers `opened`, just published, or what its handle holds once it caught up with a change published since the
  /// search it read was taken; see [`Session::catch_up`].
  pub fn catch_up(&self, opened: Arc<SessionSnapshot<T>>, assets: &AssetMountState) -> Arc<SessionSnapshot<T>> {
    match self.find_handle(opened.session_id) {
      Some(handle) => handle.catch_up(opened, assets),
      None => opened,
    }
  }

  /// Closes the caller's handles, committed or pending, and only those.
  pub fn close(&self, ids: &[SessionId]) -> TauriResult<()> {
    drop(self.detach(ids)?);

    Ok(())
  }

  /// Closes the caller's handles, handing over what they held, so a caller can drop a large value on its pool.
  pub fn detach(&self, ids: &[SessionId]) -> TauriResult<Vec<Arc<SessionSnapshot<T>>>> {
    let handles: Vec<(SessionId, Session<T>)> = {
      let mut state: MutexGuard<'_, SessionsState<T>> = self.lock();

      ids
        .iter()
        .filter_map(|id| {
          state.committed.remove(id);
          state.handles.remove(id).map(|handle| (*id, handle))
        })
        .collect()
    };
    let mut detached: Vec<Arc<SessionSnapshot<T>>> = Vec::new();

    // Each handle is closed as well as forgotten, so a derivation of it landing later publishes nothing.
    for (id, handle) in handles {
      detached.extend(handle.detach(&[id])?);
    }

    Ok(detached)
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
  fn follow_in(&self, assets: &AssetMountState, fan_out: &AssetFanOut) -> TauriResult<Vec<SessionChange>> {
    // Copied out, so a derivation reading inside the follow never runs under the handles' lock.
    let handles: Vec<Session<T>> = self.lock().handles.values().cloned().collect();
    let mut changes: Vec<SessionChange> = Vec::new();

    for handle in handles {
      changes.extend(handle.follow_in(assets, fan_out)?);
    }

    Ok(changes)
  }
}

impl<T: Send + Sync + 'static> SessionRelease for Sessions<T> {
  fn release(&self, ids: &[SessionId]) {
    if let Err(error) = self.close(ids) {
      log::error!("Released handles could not be closed: {error}");
    }
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use crate::session::{SessionId, SessionScope, SessionScopes, SessionSnapshot, Sessions};

  fn scope(window: &str, tool: &str) -> SessionScope {
    SessionScope {
      window: window.to_owned(),
      tool: tool.to_owned(),
    }
  }

  fn open(sessions: &Sessions<&'static str>, value: &'static str) -> SessionId {
    let id: SessionId = SessionId::generate();

    sessions.begin_open(id).expect("an open to begin");
    sessions.commit_open(id, value).expect("an open to commit");

    id
  }

  #[test]
  fn holds_every_handle_apart() {
    let sessions: Sessions<&'static str> = Sessions::new("test", &SessionScopes::default());
    let (first, second): (SessionId, SessionId) = (open(&sessions, "first"), open(&sessions, "second"));

    assert_eq!(**sessions.require(first).unwrap(), "first");
    assert_eq!(**sessions.require(second).unwrap(), "second");

    sessions.close(&[first]).unwrap();

    assert!(sessions.require(first).is_err(), "a closed handle answers nothing");
    assert_eq!(**sessions.require(second).unwrap(), "second", "another handle stays");
  }

  #[test]
  fn refuses_to_publish_a_handle_closed_while_it_opened() {
    let sessions: Sessions<&'static str> = Sessions::new("test", &SessionScopes::default());
    let id: SessionId = SessionId::generate();

    sessions.begin_open(id).unwrap();
    sessions.close(&[id]).unwrap();

    assert!(sessions.commit_open(id, "late").is_err());
    assert!(sessions.require(id).is_err());
  }

  #[test]
  fn restores_the_newest_opening_of_a_scope() {
    let sessions: Sessions<&'static str> = Sessions::new("test", &SessionScopes::default());
    let (older, newer, other): (SessionId, SessionId, SessionId) = (
      open(&sessions, "older"),
      open(&sessions, "newer"),
      open(&sessions, "other"),
    );
    let restored: Option<Arc<SessionSnapshot<&'static str>>> = sessions.restore(&[newer, older]).unwrap();

    assert_eq!(restored.map(|it| **it), Some("newer"));
    assert_eq!(sessions.restore(&[other]).unwrap().map(|it| **it), Some("other"));
    assert!(sessions.restore(&[]).unwrap().is_none());

    sessions.close(&[newer]).unwrap();

    assert_eq!(
      sessions.restore(&[newer, older]).unwrap().map(|it| **it),
      Some("older"),
      "a closed handle gives way to the newest still open"
    );
  }

  #[test]
  fn a_released_scope_closes_only_its_own_handles_in_every_session() {
    let scopes: SessionScopes = SessionScopes::default();
    let levels: Sessions<&'static str> = Sessions::new("levels", &scopes);
    let models: Sessions<&'static str> = Sessions::new("models", &scopes);
    let (viewer, editor): (SessionScope, SessionScope) = (scope("main", "level-viewer"), scope("main", "level-editor"));
    let (level, model, kept): (SessionId, SessionId, SessionId) =
      (SessionId::generate(), SessionId::generate(), SessionId::generate());

    scopes.bind(level, viewer.clone());
    scopes.bind(model, viewer.clone());
    scopes.bind(kept, editor.clone());

    for (sessions, id) in [(&levels, level), (&models, model), (&levels, kept)] {
      sessions.begin_open(id).unwrap();
      sessions.commit_open(id, "open").unwrap();
    }

    assert_eq!(scopes.release(&viewer), 2);
    assert!(levels.require(level).is_err());
    assert!(models.require(model).is_err());
    assert!(levels.require(kept).is_ok(), "another tool's handle stays");
    assert!(scopes.list(&viewer).is_empty());
    assert_eq!(scopes.list(&editor), vec![kept]);
  }

  #[test]
  fn a_page_keeps_the_scopes_it_reports_live_and_a_closed_window_takes_every_one() {
    let scopes: SessionScopes = SessionScopes::default();
    let levels: Sessions<&'static str> = Sessions::new("levels", &scopes);
    let (live, left, elsewhere): (SessionScope, SessionScope, SessionScope) = (
      scope("main", "level-viewer"),
      scope("main", "level-editor"),
      scope("second", "level-viewer"),
    );
    let ids: Vec<SessionId> = [&live, &left, &elsewhere]
      .into_iter()
      .map(|bound| {
        let id: SessionId = open(&levels, "open");

        scopes.bind(id, bound.clone());

        id
      })
      .collect();

    assert_eq!(scopes.retain("main", std::slice::from_ref(&live)), 1);
    assert!(levels.require(ids[0]).is_ok(), "the live scope keeps its handle");
    assert!(levels.require(ids[1]).is_err(), "the scope no page holds is released");
    assert!(levels.require(ids[2]).is_ok(), "another window is untouched");

    assert_eq!(scopes.release_window("main"), 1);
    assert!(levels.require(ids[0]).is_err());
    assert!(levels.require(ids[2]).is_ok());
  }
}
