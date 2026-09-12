use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use super::loading::{SpawnInput, open_spawn};
use super::state::SpawnFileState;
use serde_json::json;
use uuid::Uuid;
use xrf_db::{SpawnFile, XRayByteOrder};
use xrf_job::ExecutionRequest;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::core::execution::ExecutionState;
use crate::core::session::SessionId;

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

#[test]
fn pending_replacement_keeps_the_committed_file_readable() {
  let state = SpawnFileState::new();
  let first = SessionId::new();

  state.begin_open(first).unwrap();

  let descriptor = state
    .commit_open(first, "first.spawn".into(), synthetic_spawn())
    .unwrap();
  let snapshot = state.require(first).unwrap();
  let next = SessionId::new();

  state.begin_open(next).unwrap();

  assert_eq!(state.get_descriptor().unwrap().unwrap().session_id, first);
  assert!(Arc::ptr_eq(&snapshot, &state.require(first).unwrap()));
  assert_eq!(descriptor.path, PathBuf::from("first.spawn"));
  assert_eq!(descriptor.header.guid, snapshot.file.header.guid);

  state.commit_open(next, "next.spawn".into(), synthetic_spawn()).unwrap();
  assert!(state.require(first).is_err());
  assert_eq!(
    state.require(next).unwrap().descriptor.path,
    PathBuf::from("next.spawn")
  );
  assert_eq!(snapshot.descriptor.session_id, first);
}

#[test]
fn newer_open_prevents_an_older_open_from_committing() {
  let state = SpawnFileState::new();
  let older = SessionId::new();

  state.begin_open(older).unwrap();

  let newer = SessionId::new();

  state.begin_open(newer).unwrap();

  // Reserving a newer open is sufficient, even if that newer load fails.
  assert!(state.commit_open(older, "old.spawn".into(), synthetic_spawn()).is_err());
  assert!(state.get_descriptor().unwrap().is_none());

  state.commit_open(newer, "new.spawn".into(), synthetic_spawn()).unwrap();

  assert!(state.commit_open(older, "old.spawn".into(), synthetic_spawn()).is_err());
  assert_eq!(state.get_descriptor().unwrap().unwrap().session_id, newer);
}

#[test]
fn close_invalidates_pending_opens_and_reads_but_preserves_acquired_snapshots() {
  let state = SpawnFileState::new();
  let first = SessionId::new();

  state.begin_open(first).unwrap();
  state
    .commit_open(first, "first.spawn".into(), synthetic_spawn())
    .unwrap();

  let snapshot = state.require(first).unwrap();
  let pending = SessionId::new();

  state.begin_open(pending).unwrap();
  state.close(&[first, pending]).unwrap();

  assert!(state.get_descriptor().unwrap().is_none());
  assert!(state.require(first).is_err());
  assert!(
    state
      .commit_open(pending, "late.spawn".into(), synthetic_spawn())
      .is_err()
  );
  assert_eq!(snapshot.descriptor.session_id, first);

  let reopened = SessionId::new();

  state.begin_open(reopened).unwrap();
  state
    .commit_open(reopened, "first.spawn".into(), synthetic_spawn())
    .unwrap();

  assert_ne!(first, reopened);
  assert!(state.require(first).is_err());
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

  let state = SpawnFileState::new();
  let execution = ExecutionState::new(ExecutionRequest::Auto).unwrap();

  tauri::async_runtime::block_on(async {
    for (path, input) in [(packed, SpawnInput::Packed), (unpacked, SpawnInput::Unpacked)] {
      let opened = open_spawn(SessionId::new(), path.clone(), input, &state, &execution)
        .await
        .unwrap();

      assert_eq!(opened.path, path);
      assert_eq!(
        serde_json::to_value(&state.require(opened.session_id).unwrap().file).unwrap(),
        serde_json::to_value(&source).unwrap()
      );

      for input in [SpawnInput::Packed, SpawnInput::Unpacked] {
        assert!(
          open_spawn(SessionId::new(), corrupt.clone(), input, &state, &execution)
            .await
            .is_err()
        );
        assert_eq!(state.get_descriptor().unwrap().unwrap().session_id, opened.session_id);
        assert!(state.require(opened.session_id).is_ok());
      }
    }
  });
}
