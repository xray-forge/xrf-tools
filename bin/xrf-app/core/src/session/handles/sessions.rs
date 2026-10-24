use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use rayon::ThreadPool;
use xrf_vfs::{XrayInputRecord, XrayRevision};

use crate::assets::{AssetFanOut, AssetMountState};
use crate::session::handles::sessions_state::SessionsState;
use crate::session::{
  Session, SessionBindings, SessionChange, SessionDerived, SessionFollower, SessionId, SessionOpening, SessionParts,
  SessionRelease, SessionScope, SessionScopes, SessionSnapshot,
};
use crate::types::TauriResult;

/// Every opening of one kind of document, each a handle its opener closes: the many a domain holds at once, one for each
/// tool and document. Each handle publishes its snapshots as one [`Session`] does, and is followed through commits with
/// the rest. A clone answers for the same handles, and opens new ones with the same derivation.
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

    handle.begin_open(id)?;

    if !self.bindings.is_bound(id) {
      return Err(format!(
        "The {} opening is bound to no tool, so nothing would close it",
        handle.get_name()
      ));
    }

    Ok(opening)
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
    self.bindings.bind(
      id,
      SessionScope {
        window: String::from("fixture"),
        tool: id.to_string(),
      },
    );

    self.begin_open(id)
  }

  /// Publishes a handle's opening at `revision`, with the inputs its value was read from and the parts it read on its
  /// way; see [`Session::commit_open_with`]. A handle closed as it commits is not one a restore answers.
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

  /// Lets go of a handle whose open was given up before it committed; one that committed is the opener's to close.
  pub(super) fn abandon(&self, id: SessionId) {
    let handle: Option<Session<T>> = {
      let mut state: MutexGuard<'_, SessionsState<T>> = self.lock();

      if state.committed.contains_key(&id) {
        return;
      }

      state.handles.remove(&id)
    };

    if let Some(handle) = handle
      && let Err(error) = handle.close(&[id])
    {
      log::error!("An opening given up could not be let go: {error}");
    }

    self.bindings.unbind(&[id]);
  }

  /// The snapshot a handle holds, never another's.
  pub fn require(&self, id: SessionId) -> TauriResult<Arc<SessionSnapshot<T>>> {
    self.require_handle(id)?.require(id)
  }

  /// The newest committed snapshot among the handles `scope` holds, for a page attaching to what its tool holds; none
  /// where none of them holds one.
  pub fn restore(&self, scope: &SessionScope) -> TauriResult<Option<Arc<SessionSnapshot<T>>>> {
    let ids: Vec<SessionId> = self.bindings.list(scope);
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

  /// Closes the caller's handles, committed or pending, and only those, dropping what they held on the scopes' pool. A
  /// handle whose lock was lost is said aloud and forgotten, leaving the rest to close.
  pub fn close(&self, ids: &[SessionId]) -> TauriResult<()> {
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

    // Unbound whether or not a handle was reserved yet, so an id closed before its open began opens no more.
    self.bindings.unbind(ids);

    if handles.is_empty() {
      return Ok(());
    }

    let mut detached: Vec<Arc<SessionSnapshot<T>>> = Vec::new();

    // Each handle is closed as well as forgotten, so a derivation of it landing later publishes nothing.
    for (id, handle) in handles {
      match handle.detach(&[id]) {
        Ok(snapshot) => detached.extend(snapshot),
        Err(error) => log::error!("A {} handle could not be closed: {error}", handle.get_name()),
      }
    }

    if !detached.is_empty() {
      self.bindings.dispose(detached);
    }

    Ok(())
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
  /// Follows every handle, one failing leaving the rest to move: what each answers is announced, and a handle whose
  /// lock was lost is said aloud rather than taking the others' changes with it.
  fn follow_in(&self, assets: &AssetMountState, fan_out: &AssetFanOut) -> TauriResult<Vec<SessionChange>> {
    // Copied out, so a derivation reading inside the follow never runs under the handles' lock.
    let handles: Vec<Session<T>> = self.lock().handles.values().cloned().collect();
    let mut changes: Vec<SessionChange> = Vec::new();

    for handle in handles {
      match handle.follow_in(assets, fan_out) {
        Ok(change) => changes.extend(change),
        Err(error) => log::error!("A {} handle could not follow a commit: {error}", handle.get_name()),
      }
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
  use std::sync::mpsc;
  use std::sync::{Arc, Weak};
  use std::thread;
  use std::time::{Duration, Instant};

  use rayon::{ThreadPool, ThreadPoolBuilder};

  use crate::session::{SessionId, SessionScope, SessionScopes, SessionSnapshot, Sessions};

  fn scope(window: &str, tool: &str) -> SessionScope {
    SessionScope {
      window: window.to_owned(),
      tool: tool.to_owned(),
    }
  }

  /// Opens `value` as one more handle bound to `bound`, as a tool's open does.
  fn open(
    sessions: &Sessions<&'static str>,
    scopes: &SessionScopes,
    bound: &SessionScope,
    value: &'static str,
  ) -> SessionId {
    let id: SessionId = SessionId::generate();

    scopes.bind(id, bound.clone());
    sessions
      .begin_open(id)
      .expect("an open to begin")
      .commit(value)
      .expect("an open to commit");

    id
  }

  #[test]
  fn holds_every_handle_apart() {
    let scopes: SessionScopes = SessionScopes::default();
    let sessions: Sessions<&'static str> = Sessions::new("test", &scopes);
    let tool: SessionScope = scope("main", "tool");
    let (first, second): (SessionId, SessionId) = (
      open(&sessions, &scopes, &tool, "first"),
      open(&sessions, &scopes, &tool, "second"),
    );

    assert_eq!(**sessions.require(first).unwrap(), "first");
    assert_eq!(**sessions.require(second).unwrap(), "second");

    sessions.close(&[first]).unwrap();

    assert!(sessions.require(first).is_err(), "a closed handle answers nothing");
    assert_eq!(**sessions.require(second).unwrap(), "second", "another handle stays");
    assert_eq!(scopes.list(&tool), vec![second], "a closed handle is unbound");
  }

  #[test]
  fn refuses_to_open_a_handle_no_scope_holds() {
    let sessions: Sessions<&'static str> = Sessions::new("test", &SessionScopes::default());

    assert!(sessions.begin_open(SessionId::generate()).is_err());
  }

  #[test]
  fn refuses_to_publish_a_handle_closed_while_it_opened() {
    let sessions: Sessions<&'static str> = Sessions::new("test", &SessionScopes::default());
    let id: SessionId = SessionId::generate();
    let opening = sessions.begin_fixture_open(id).unwrap();

    sessions.close(&[id]).unwrap();

    assert!(opening.commit("late").is_err());
    assert!(sessions.require(id).is_err());
  }

  #[test]
  fn an_open_given_up_leaves_no_handle_and_no_binding() {
    let scopes: SessionScopes = SessionScopes::default();
    let sessions: Sessions<&'static str> = Sessions::new("test", &scopes);
    let tool: SessionScope = scope("main", "tool");
    let kept: SessionId = open(&sessions, &scopes, &tool, "kept");
    let failed: SessionId = SessionId::generate();

    scopes.bind(failed, tool.clone());
    drop(sessions.begin_open(failed).unwrap());

    assert!(sessions.require(failed).is_err());
    assert_eq!(scopes.list(&tool), vec![kept], "only the committed handle stays bound");
    assert_eq!(sessions.list_opened().unwrap().len(), 1);
  }

  #[test]
  fn restores_the_newest_opening_of_a_scope() {
    let scopes: SessionScopes = SessionScopes::default();
    let sessions: Sessions<&'static str> = Sessions::new("test", &scopes);
    let (tool, other_tool): (SessionScope, SessionScope) = (scope("main", "tool"), scope("main", "other"));
    let _older: SessionId = open(&sessions, &scopes, &tool, "older");
    let newer: SessionId = open(&sessions, &scopes, &tool, "newer");
    let _other: SessionId = open(&sessions, &scopes, &other_tool, "other");
    let restored: Option<Arc<SessionSnapshot<&'static str>>> = sessions.restore(&tool).unwrap();

    assert_eq!(restored.map(|it| **it), Some("newer"));
    assert_eq!(sessions.restore(&other_tool).unwrap().map(|it| **it), Some("other"));
    assert!(sessions.restore(&scope("main", "none")).unwrap().is_none());

    sessions.close(&[newer]).unwrap();

    assert_eq!(
      sessions.restore(&tool).unwrap().map(|it| **it),
      Some("older"),
      "a closed handle gives way to the newest still open"
    );
  }

  #[test]
  fn a_closed_document_is_dropped_on_the_scopes_pool() {
    let pool: Arc<ThreadPool> = Arc::new(ThreadPoolBuilder::new().num_threads(1).build().unwrap());
    let scopes: SessionScopes = SessionScopes::new(Arc::clone(&pool));
    let sessions: Sessions<Vec<u8>> = Sessions::new("test", &scopes);
    let id: SessionId = SessionId::generate();
    let (park, parked) = mpsc::channel::<()>();

    sessions.begin_fixture_open(id).unwrap().commit(vec![1, 2, 3]).unwrap();

    let held: Weak<SessionSnapshot<Vec<u8>>> = Arc::downgrade(&sessions.require(id).unwrap());

    // The pool's one worker waits, so the document can only be dropped once it is let go.
    pool.spawn(move || parked.recv().unwrap());
    sessions.close(&[id]).unwrap();

    assert!(held.upgrade().is_some(), "the close left the drop to the pool");

    park.send(()).unwrap();

    let started: Instant = Instant::now();

    while held.upgrade().is_some() {
      assert!(
        started.elapsed() < Duration::from_secs(10),
        "the pool dropped the document"
      );
      thread::yield_now();
    }
  }

  #[test]
  fn a_released_scope_closes_only_its_own_handles_in_every_session() {
    let scopes: SessionScopes = SessionScopes::default();
    let levels: Sessions<&'static str> = Sessions::new("levels", &scopes);
    let models: Sessions<&'static str> = Sessions::new("models", &scopes);
    let (viewer, editor): (SessionScope, SessionScope) = (scope("main", "level-viewer"), scope("main", "level-editor"));
    let level: SessionId = open(&levels, &scopes, &viewer, "open");
    let model: SessionId = open(&models, &scopes, &viewer, "open");
    let kept: SessionId = open(&levels, &scopes, &editor, "open");

    assert_eq!(scopes.release(&viewer), 2);
    assert!(levels.require(level).is_err());
    assert!(models.require(model).is_err());
    assert!(levels.require(kept).is_ok(), "another tool's handle stays");
    assert!(scopes.list(&viewer).is_empty());
    assert_eq!(scopes.list(&editor), vec![kept]);
  }

  #[test]
  fn a_scope_released_while_its_handle_opens_refuses_the_commit() {
    let scopes: SessionScopes = SessionScopes::default();
    let levels: Sessions<&'static str> = Sessions::new("levels", &scopes);
    let viewer: SessionScope = scope("main", "level-viewer");
    let id: SessionId = SessionId::generate();

    scopes.bind(id, viewer.clone());

    let opening = levels.begin_open(id).unwrap();

    assert_eq!(scopes.release(&viewer), 1);
    assert!(opening.commit("late").is_err(), "the tool was left while it opened");
    assert!(levels.require(id).is_err());
    assert!(levels.begin_open(id).is_err(), "an unbound handle opens no more");
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
      .map(|bound| open(&levels, &scopes, bound, "open"))
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
