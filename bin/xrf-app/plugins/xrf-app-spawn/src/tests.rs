use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use serde_json::json;
use uuid::Uuid;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionScope, SessionScopes};
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

/// A spawn editor's openings: the state, the scopes they are bound in, and the editor's own scope.
struct SpawnTool {
  state: SpawnFileState,
  scopes: SessionScopes,
  scope: SessionScope,
}

impl SpawnTool {
  fn new() -> Self {
    let scopes: SessionScopes = SessionScopes::default();

    Self {
      state: SpawnFileState::new(&scopes),
      scopes,
      scope: SessionScope {
        window: String::from("main"),
        tool: String::from("spawn-editor"),
      },
    }
  }

  /// An id the editor bound before opening it, as its page does.
  fn bind(&self) -> SessionId {
    let id: SessionId = SessionId::generate();

    self.scopes.bind(id, self.scope.clone());

    id
  }

  /// The id of the spawn a restore answers.
  fn restored(&self) -> Option<SessionId> {
    self
      .state
      .restore_descriptor(&self.scope)
      .unwrap()
      .map(|descriptor| descriptor.session_id)
  }
}

/// Each open is a handle of its own, which its opener closes once a newer one replaces it: a pending open leaves the one
/// committed readable, and a restore answers the newest committed.
#[test]
fn an_open_beside_another_keeps_it_readable_and_a_restore_answers_the_newest() {
  let tool: SpawnTool = SpawnTool::new();
  let first = tool.bind();
  let descriptor = SpawnFileState::commit_open(
    tool.state.files.begin_open(first).unwrap(),
    "first.spawn".into(),
    synthetic_spawn(),
  )
  .unwrap();
  let snapshot = tool.state.files.require(first).unwrap();
  let next = tool.bind();
  let opening = tool.state.files.begin_open(next).unwrap();

  assert_eq!(tool.restored(), Some(first));
  assert!(Arc::ptr_eq(&snapshot, &tool.state.files.require(first).unwrap()));
  assert_eq!(descriptor.path, PathBuf::from("first.spawn"));
  assert_eq!(descriptor.header.guid, snapshot.file.header.guid);

  SpawnFileState::commit_open(opening, "next.spawn".into(), synthetic_spawn()).unwrap();

  assert!(
    tool.state.files.require(first).is_ok(),
    "the first stays until its opener closes it"
  );
  assert_eq!(tool.restored(), Some(next));
  assert_eq!(
    tool.state.files.require(next).unwrap().descriptor.path,
    PathBuf::from("next.spawn")
  );
}

/// A handle closed while it opened publishes nothing, whatever opens beside it.
#[test]
fn a_handle_closed_while_it_opened_commits_nothing() {
  let tool: SpawnTool = SpawnTool::new();
  let (older, newer) = (tool.bind(), tool.bind());
  let older_opening = tool.state.files.begin_open(older).unwrap();
  let newer_opening = tool.state.files.begin_open(newer).unwrap();

  tool.state.files.close(&[older]).unwrap();

  assert!(SpawnFileState::commit_open(older_opening, "old.spawn".into(), synthetic_spawn()).is_err());

  SpawnFileState::commit_open(newer_opening, "new.spawn".into(), synthetic_spawn()).unwrap();

  assert_eq!(tool.restored(), Some(newer));
}

#[test]
fn close_invalidates_pending_opens_and_reads_but_preserves_acquired_snapshots() {
  let tool: SpawnTool = SpawnTool::new();
  let first = tool.bind();

  SpawnFileState::commit_open(
    tool.state.files.begin_open(first).unwrap(),
    "first.spawn".into(),
    synthetic_spawn(),
  )
  .unwrap();

  let snapshot = tool.state.files.require(first).unwrap();
  let pending = tool.bind();
  let opening = tool.state.files.begin_open(pending).unwrap();

  tool.state.files.close(&[first, pending]).unwrap();

  assert_eq!(tool.restored(), None);
  assert!(tool.state.files.require(first).is_err());
  assert!(SpawnFileState::commit_open(opening, "late.spawn".into(), synthetic_spawn()).is_err());
  assert_eq!(snapshot.descriptor.session_id, first);

  let reopened = tool.bind();

  SpawnFileState::commit_open(
    tool.state.files.begin_open(reopened).unwrap(),
    "first.spawn".into(),
    synthetic_spawn(),
  )
  .unwrap();

  assert_ne!(first, reopened);
  assert!(tool.state.files.require(first).is_err());
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

  let tool: SpawnTool = SpawnTool::new();
  let execution = ExecutionState::new(ExecutionRequest::Auto).unwrap();

  tauri::async_runtime::block_on(async {
    for (path, input) in [(packed, SpawnInput::Packed), (unpacked, SpawnInput::Unpacked)] {
      let opened = open_spawn(tool.bind(), path.clone(), input, &tool.state, &execution)
        .await
        .unwrap();

      assert_eq!(opened.path, path);
      assert_eq!(
        serde_json::to_value(&tool.state.files.require(opened.session_id).unwrap().file).unwrap(),
        serde_json::to_value(&source).unwrap()
      );

      for input in [SpawnInput::Packed, SpawnInput::Unpacked] {
        let failed: SessionId = tool.bind();

        assert!(
          open_spawn(failed, corrupt.clone(), input, &tool.state, &execution)
            .await
            .is_err()
        );
        assert_eq!(tool.restored(), Some(opened.session_id));
        assert!(tool.state.files.require(opened.session_id).is_ok());
        assert!(
          !tool.scopes.list(&tool.scope).contains(&failed),
          "a failed open leaves no binding behind"
        );
      }
    }
  });
}
