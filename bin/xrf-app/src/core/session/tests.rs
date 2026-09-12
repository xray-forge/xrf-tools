use std::sync::Arc;

use crate::core::session::{Session, SessionId, SessionRestore, SessionSnapshot};

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
