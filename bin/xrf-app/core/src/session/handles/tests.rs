use std::sync::mpsc;
use std::sync::{Arc, Weak};
use std::thread;
use std::time::{Duration, Instant};

use rayon::{ThreadPool, ThreadPoolBuilder};

use crate::session::{SessionId, SessionScope, SessionScopes, SessionSnapshot, Sessions};

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
  let tool: SessionScope = SessionScope::new("main", "tool");
  let (first, second): (SessionId, SessionId) = (
    open(&sessions, &scopes, &tool, "first"),
    open(&sessions, &scopes, &tool, "second"),
  );

  assert_eq!(**sessions.require(first).unwrap(), "first");
  assert_eq!(**sessions.require(second).unwrap(), "second");

  sessions.close(&[first]);

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

  sessions.close(&[id]);

  assert!(opening.commit("late").is_err());
  assert!(sessions.require(id).is_err());
}

#[test]
fn an_open_given_up_leaves_no_handle_and_no_binding() {
  let scopes: SessionScopes = SessionScopes::default();
  let sessions: Sessions<&'static str> = Sessions::new("test", &scopes);
  let tool: SessionScope = SessionScope::new("main", "tool");
  let kept: SessionId = open(&sessions, &scopes, &tool, "kept");
  let failed: SessionId = SessionId::generate();

  scopes.bind(failed, tool.clone());
  drop(sessions.begin_open(failed).unwrap());

  assert!(sessions.require(failed).is_err());
  assert_eq!(scopes.list(&tool), vec![kept], "only the committed handle stays bound");
  assert_eq!(sessions.list_opened().len(), 1);
}

/// Giving up a second open of a committed handle forgets only that open: the handle keeps what it holds, its binding,
/// and the next open of it publishes.
#[test]
fn a_reopen_given_up_keeps_the_committed_handle() {
  let scopes: SessionScopes = SessionScopes::default();
  let sessions: Sessions<&'static str> = Sessions::new("test", &scopes);
  let tool: SessionScope = SessionScope::new("main", "tool");
  let id: SessionId = open(&sessions, &scopes, &tool, "committed");

  drop(sessions.begin_open(id).unwrap());

  assert_eq!(**sessions.require(id).unwrap(), "committed");
  assert_eq!(scopes.list(&tool), vec![id]);

  sessions.begin_open(id).unwrap().commit("reopened").unwrap();

  assert_eq!(**sessions.require(id).unwrap(), "reopened");
}

#[test]
fn a_reload_answers_the_previous_snapshot_and_opens_a_new_handle() {
  let scopes: SessionScopes = SessionScopes::default();
  let sessions: Sessions<&'static str> = Sessions::new("test", &scopes);
  let tool: SessionScope = SessionScope::new("main", "tool");
  let previous: SessionId = open(&sessions, &scopes, &tool, "previous");
  let reloaded: SessionId = SessionId::generate();

  scopes.bind(reloaded, tool.clone());

  let (opening, held) = sessions.begin_reload(reloaded, previous).unwrap();

  assert_eq!(**held, "previous");
  assert_eq!(opening.get_id(), reloaded);

  opening.commit("reloaded").unwrap();

  assert_eq!(
    **sessions.require(previous).unwrap(),
    "previous",
    "the opener closes the previous handle"
  );
  assert_eq!(**sessions.require(reloaded).unwrap(), "reloaded");
  assert!(
    sessions
      .begin_reload(SessionId::generate(), SessionId::generate())
      .is_err()
  );
}

#[test]
fn restores_the_newest_opening_of_a_scope() {
  let scopes: SessionScopes = SessionScopes::default();
  let sessions: Sessions<&'static str> = Sessions::new("test", &scopes);
  let (tool, other_tool): (SessionScope, SessionScope) =
    (SessionScope::new("main", "tool"), SessionScope::new("main", "other"));
  let _older: SessionId = open(&sessions, &scopes, &tool, "older");
  let newer: SessionId = open(&sessions, &scopes, &tool, "newer");
  let _other: SessionId = open(&sessions, &scopes, &other_tool, "other");
  let restored: Option<Arc<SessionSnapshot<&'static str>>> = sessions.restore(&tool);

  assert_eq!(restored.map(|it| **it), Some("newer"));
  assert_eq!(sessions.restore(&other_tool).map(|it| **it), Some("other"));
  assert!(sessions.restore(&SessionScope::new("main", "none")).is_none());

  sessions.close(&[newer]);

  assert_eq!(
    sessions.restore(&tool).map(|it| **it),
    Some("older"),
    "a closed handle gives way to the newest still open"
  );
}

#[test]
fn restores_a_description_under_the_snapshot_identity() {
  let scopes: SessionScopes = SessionScopes::default();
  let sessions: Sessions<&'static str> = Sessions::new("test", &scopes);
  let tool: SessionScope = SessionScope::new("main", "tool");
  let id: SessionId = open(&sessions, &scopes, &tool, "value");
  let restored: SessionSnapshot<usize> = sessions.restore_as(&tool, |value| value.len()).unwrap();

  assert_eq!(restored.session_id, id);
  assert_eq!(*restored, 5);
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
  sessions.close(&[id]);

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
  let (viewer, editor): (SessionScope, SessionScope) = (
    SessionScope::new("main", "level-viewer"),
    SessionScope::new("main", "level-editor"),
  );
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
  let viewer: SessionScope = SessionScope::new("main", "level-viewer");
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
    SessionScope::new("main", "level-viewer"),
    SessionScope::new("main", "level-editor"),
    SessionScope::new("second", "level-viewer"),
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
