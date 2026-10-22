use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use rayon::prelude::*;

use crate::core::session::{Session, SessionId, SessionPart, SessionParts, SessionRestore, SessionSnapshot};

#[test]
fn a_failed_replacement_keeps_the_committed_value_readable() {
  let session: Session<String> = Session::new("test");
  let first: SessionId = SessionId::new();

  session.begin_open(first).unwrap();
  session.commit_open(first, String::from("first")).unwrap();
  session.begin_open(SessionId::new()).unwrap();

  assert_eq!(session.require(first).unwrap().as_str(), "first");
}

#[test]
fn a_late_open_cannot_replace_a_newer_open_or_undo_its_close() {
  let session: Session<&str> = Session::new("test");
  let first: SessionId = SessionId::new();
  let second: SessionId = SessionId::new();

  session.begin_open(first).unwrap();
  session.begin_open(second).unwrap();
  session.commit_open(second, "second").unwrap();

  assert!(session.commit_open(first, "first").is_err());

  session.close(&[second]).unwrap();

  assert!(session.commit_open(first, "first").is_err());
  assert!(session.get().unwrap().is_none());
}

#[test]
fn closing_a_pending_open_prevents_its_publication() {
  let session: Session<&str> = Session::new("test");
  let id: SessionId = SessionId::new();

  session.begin_open(id).unwrap();
  session.close(&[id]).unwrap();

  assert!(session.commit_open(id, "late").is_err());
}

#[test]
fn an_old_teardown_cannot_close_a_new_owners_session_or_opening() {
  let session: Session<&str> = Session::new("test");
  let old: SessionId = SessionId::new();
  let new: SessionId = SessionId::new();

  session.begin_open(old).unwrap();
  session.commit_open(old, "old").unwrap();
  session.begin_open(new).unwrap();
  session.close(&[old]).unwrap();
  session.commit_open(new, "new").unwrap();
  session.close(&[old]).unwrap();

  assert_eq!(**session.require(new).unwrap(), "new");
  assert!(session.require(old).is_err());
}

#[test]
fn an_accepted_snapshot_outlives_close_and_a_stale_save_cannot_publish() {
  let session: Session<String> = Session::new("test");
  let id: SessionId = SessionId::new();

  session.begin_open(id).unwrap();

  let held: Arc<SessionSnapshot<String>> = session.commit_open(id, String::from("before")).unwrap();
  let saved = session.replace(&held, String::from("saved")).unwrap().unwrap();

  assert!(session.replace(&held, String::from("outdated save")).unwrap().is_none());

  session.close(&[id]).unwrap();

  assert_eq!(held.as_str(), "before");
  assert_eq!(saved.as_str(), "saved");
  assert!(session.replace(&saved, String::from("after close")).unwrap().is_none());
}

#[test]
fn restoration_and_open_serialize_the_same_identity_and_value() {
  let session: Session<Vec<u8>> = Session::new("wire test");
  let id: SessionId = SessionId::new();

  session.begin_open(id).unwrap();

  let opened = session.commit_open(id, vec![1, 2, 3]).unwrap();
  let restored = SessionRestore::from(session.get().unwrap());
  let expected = serde_json::json!({ "sessionId": id, "value": [1, 2, 3] });

  assert_eq!(serde_json::to_value(&opened).unwrap(), expected);
  assert_eq!(serde_json::to_value(&restored).unwrap(), expected);

  session.close(&[id]).unwrap();

  assert_eq!(
    serde_json::to_value(SessionRestore::from(session.get().unwrap())).unwrap(),
    serde_json::Value::Null
  );
}

#[test]
fn reloading_checks_the_previous_identity_before_reserving_publication() {
  let session: Session<&str> = Session::new("reload test");
  let first: SessionId = SessionId::new();
  let second: SessionId = SessionId::new();

  session.begin_open(first).unwrap();
  session.commit_open(first, "first").unwrap();
  session.begin_reload(second, first).unwrap();

  assert_eq!(**session.require(first).unwrap(), "first");
  assert!(session.begin_reload(SessionId::new(), SessionId::new()).is_err());

  session.commit_open(second, "reloaded").unwrap();

  assert!(session.require(first).is_err());
}

// Long work asks before it starts as well as before it publishes: a sector pack that has queued behind others is
// usually answering a question nobody is asking any more, and packing anyway is how a queue grows faster than it
// drains.
#[test]
fn a_superseded_opening_can_be_told_before_its_work_is_done() {
  let session: Session<&str> = Session::new("test");
  let first: SessionId = SessionId::new();
  let second: SessionId = SessionId::new();

  session.begin_open(first).unwrap();

  assert!(session.require_opening(first).is_ok());

  session.begin_open(second).unwrap();

  assert!(session.require_opening(first).is_err());
  assert!(session.require_opening(second).is_ok());
}

#[test]
fn an_opening_that_was_closed_or_published_is_no_longer_the_one_to_do_work_for() {
  let session: Session<&str> = Session::new("test");
  let id: SessionId = SessionId::new();

  session.begin_open(id).unwrap();
  session.commit_open(id, "value").unwrap();

  assert!(
    session.require_opening(id).is_err(),
    "a published opening is finished, not pending"
  );

  let closed: SessionId = SessionId::new();

  session.begin_open(closed).unwrap();
  session.close(&[closed]).unwrap();

  assert!(session.require_opening(closed).is_err());
}

#[test]
fn the_early_check_and_the_commit_refuse_a_superseded_opening_the_same_way() {
  let session: Session<&str> = Session::new("test");
  let first: SessionId = SessionId::new();

  session.begin_open(first).unwrap();
  session.begin_open(SessionId::new()).unwrap();

  let early: String = session.require_opening(first).unwrap_err();
  let late: String = session.commit_open(first, "first").unwrap_err();

  assert_eq!(early, late, "a caller cannot tell the two apart and should not have to");
}

/// A part each test builds, or keeps, by its own count, behind a plain lock.
struct CountedPart;

impl SessionPart for CountedPart {
  type Value = Arc<u32>;

  const IS_POOLED: bool = false;
}

/// A pooled part asking for another as it builds.
struct DependentPart;

impl SessionPart for DependentPart {
  type Value = u32;
}

/// A part built over Rayon while the pool's workers wait on it, pooled as parts are by default.
struct PooledPart;

impl SessionPart for PooledPart {
  type Value = u64;
}

#[test]
fn a_part_is_built_once_and_shared_by_every_ask() {
  let parts: SessionParts = SessionParts::new();
  let builds: AtomicU32 = AtomicU32::new(0);

  assert_eq!(
    parts.get::<CountedPart>(),
    None,
    "asking whether it was built builds nothing"
  );

  let first: Arc<u32> = parts.get_or_init::<CountedPart>(|| Arc::new(builds.fetch_add(1, Ordering::SeqCst) + 7));
  let second: Arc<u32> = parts.get_or_init::<CountedPart>(|| Arc::new(builds.fetch_add(1, Ordering::SeqCst)));

  assert!(Arc::ptr_eq(&first, &second));
  assert_eq!(builds.load(Ordering::SeqCst), 1);
  assert_eq!(parts.get::<CountedPart>().as_deref(), Some(&7));
}

#[test]
fn a_part_the_opening_read_is_kept_rather_than_built() {
  let parts: SessionParts = SessionParts::new()
    .with::<CountedPart>(Arc::new(3))
    .with::<PooledPart>(5);

  assert_eq!(
    *parts.get_or_init::<CountedPart>(|| unreachable!("the part was kept")),
    3
  );
  assert_eq!(parts.get::<PooledPart>(), Some(5));
  assert_eq!(
    parts.get_or_init::<PooledPart>(|| unreachable!("the pooled part was kept")),
    5
  );
}

#[test]
fn a_part_may_ask_for_another_as_it_builds() {
  let parts: SessionParts = SessionParts::new();
  let value: u32 = parts.get_or_init::<DependentPart>(|| *parts.get_or_init::<CountedPart>(|| Arc::new(2)) * 10);

  assert_eq!(value, 20);
}

#[test]
fn a_pooled_part_built_over_rayon_does_not_deadlock_the_workers_waiting_on_it() {
  let parts: SessionParts = SessionParts::new();
  let sums: Vec<u64> = (0..64)
    .into_par_iter()
    .map(|_| parts.get_or_init::<PooledPart>(|| (0..10_000u64).into_par_iter().sum()))
    .collect();

  assert!(sums.iter().all(|sum| *sum == 49_995_000));
}

#[test]
fn a_replacement_starts_without_the_parts_of_the_value_it_replaces() {
  let session: Session<&str> = Session::new("test");
  let id: SessionId = SessionId::new();

  session.begin_open(id).unwrap();

  let opened: Arc<SessionSnapshot<&str>> = session
    .commit_open_with(id, "first", SessionParts::new().with::<CountedPart>(Arc::new(1)))
    .unwrap();
  let replaced: Arc<SessionSnapshot<&str>> = session.replace(&opened, "second").unwrap().unwrap();

  assert_eq!(opened.parts.get::<CountedPart>().as_deref(), Some(&1));
  assert_eq!(replaced.parts.get::<CountedPart>(), None);
}
