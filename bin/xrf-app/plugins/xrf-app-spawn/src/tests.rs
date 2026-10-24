use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use serde_json::json;
use uuid::Uuid;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionScopes};
use xrf_job::ExecutionRequest;
use xrf_spawn::{SpawnFile, XRayByteOrder};
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use super::loading::{SpawnInput, open_spawn};
use super::state::SpawnFileState;

pub(super) fn synthetic_spawn() -> SpawnFile {
  serde_json::from_value(json!({
    "header": { "version": 10, "guid": Uuid::nil(), "graphGuid": Uuid::nil(), "objectsCount": 0, "levelsCount": 0 },
    "alifeSpawn": { "objects": [] }, "artefactSpawn": { "nodes": [] }, "patrols": { "patrols": [] },
    "graphs": {
      "header": { "version": 8, "verticesCount": 0, "edgesCount": 0, "pointsCount": 0, "guid": Uuid::nil(), "levelsCount": 0 },
      "levels": [], "vertices": [], "edges": [], "points": [],
      "crossTables": [{ "version": 8, "nodesCount": 0, "verticesCount": 0, "levelGuid": Uuid::nil(), "gameGuid": Uuid::nil() }]
    }
  })).expect("synthetic spawn with an empty cross-table")
}

/// Each open is a handle of its own, which its opener closes once a newer one replaces it: a pending open leaves the one
/// committed readable, and a restore answers the newest committed.
#[test]
fn an_open_beside_another_keeps_it_readable_and_a_restore_answers_the_newest() {
  let state = SpawnFileState::new(&SessionScopes::default());
  let first = SessionId::generate();

  state.files.begin_open(first).unwrap();

  let descriptor = state
    .commit_open(first, "first.spawn".into(), synthetic_spawn())
    .unwrap();
  let snapshot = state.files.require(first).unwrap();
  let next = SessionId::generate();

  state.files.begin_open(next).unwrap();

  assert_eq!(
    state.restore_descriptor(&[first, next]).unwrap().unwrap().session_id,
    first
  );
  assert!(Arc::ptr_eq(&snapshot, &state.files.require(first).unwrap()));
  assert_eq!(descriptor.path, PathBuf::from("first.spawn"));
  assert_eq!(descriptor.header.guid, snapshot.file.header.guid);

  state.commit_open(next, "next.spawn".into(), synthetic_spawn()).unwrap();

  assert!(
    state.files.require(first).is_ok(),
    "the first stays until its opener closes it"
  );
  assert_eq!(
    state.restore_descriptor(&[first, next]).unwrap().unwrap().session_id,
    next
  );
  assert_eq!(
    state.files.require(next).unwrap().descriptor.path,
    PathBuf::from("next.spawn")
  );
}

/// A handle closed while it opened publishes nothing, whatever opens beside it.
#[test]
fn a_handle_closed_while_it_opened_commits_nothing() {
  let state = SpawnFileState::new(&SessionScopes::default());
  let older = SessionId::generate();
  let newer = SessionId::generate();

  state.files.begin_open(older).unwrap();
  state.files.begin_open(newer).unwrap();
  state.files.close(&[older]).unwrap();

  assert!(state.commit_open(older, "old.spawn".into(), synthetic_spawn()).is_err());

  state.commit_open(newer, "new.spawn".into(), synthetic_spawn()).unwrap();

  assert_eq!(
    state.restore_descriptor(&[older, newer]).unwrap().unwrap().session_id,
    newer
  );
}

#[test]
fn close_invalidates_pending_opens_and_reads_but_preserves_acquired_snapshots() {
  let state = SpawnFileState::new(&SessionScopes::default());
  let first = SessionId::generate();

  state.files.begin_open(first).unwrap();
  state
    .commit_open(first, "first.spawn".into(), synthetic_spawn())
    .unwrap();

  let snapshot = state.files.require(first).unwrap();
  let pending = SessionId::generate();

  state.files.begin_open(pending).unwrap();
  state.files.detach(&[first, pending]).unwrap();

  assert!(state.restore_descriptor(&[first, pending]).unwrap().is_none());
  assert!(state.files.require(first).is_err());
  assert!(
    state
      .commit_open(pending, "late.spawn".into(), synthetic_spawn())
      .is_err()
  );
  assert_eq!(snapshot.descriptor.session_id, first);

  let reopened = SessionId::generate();

  state.files.begin_open(reopened).unwrap();
  state
    .commit_open(reopened, "first.spawn".into(), synthetic_spawn())
    .unwrap();

  assert_ne!(first, reopened);
  assert!(state.files.require(first).is_err());
}

#[test]
fn packed_and_unpacked_opens_preserve_the_session_when_a_replacement_fails() {
  let directory = build_absolute_generated_test_resource_path("spawn/session/loading");
  fs::create_dir_all(&directory).unwrap();
  let packed = directory.join("all.spawn");
  let unpacked = directory.join("unpacked");
  let corrupt = directory.join("corrupt.spawn");
  let source = synthetic_spawn();
  source.write_to_path::<XRayByteOrder, _>(&packed).unwrap();
  source.export_to_path::<XRayByteOrder, _>(&unpacked).unwrap();
  fs::write(&corrupt, b"invalid spawn").unwrap();

  let state = SpawnFileState::new(&SessionScopes::default());
  let execution = ExecutionState::new(ExecutionRequest::Auto).unwrap();

  tauri::async_runtime::block_on(async {
    for (path, input) in [(packed, SpawnInput::Packed), (unpacked, SpawnInput::Unpacked)] {
      let opened = open_spawn(SessionId::generate(), path.clone(), input, &state, &execution)
        .await
        .unwrap();

      assert_eq!(opened.path, path);
      assert_eq!(
        serde_json::to_value(&state.files.require(opened.session_id).unwrap().file).unwrap(),
        serde_json::to_value(&source).unwrap()
      );

      for input in [SpawnInput::Packed, SpawnInput::Unpacked] {
        assert!(
          open_spawn(SessionId::generate(), corrupt.clone(), input, &state, &execution)
            .await
            .is_err()
        );
        assert_eq!(
          state
            .restore_descriptor(&[opened.session_id])
            .unwrap()
            .unwrap()
            .session_id,
          opened.session_id
        );
        assert!(state.files.require(opened.session_id).is_ok());
      }
    }
  });
}
