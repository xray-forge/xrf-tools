use std::fs;
use std::path::PathBuf;

use serde_json::json;
use uuid::Uuid;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionScope, SessionScopes};
use xrf_chunk::XRayByteOrder;
use xrf_job::ExecutionRequest;
use xrf_spawn::SpawnFile;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::{SpawnChunk, SpawnChunkKind, SpawnFiles, SpawnSession};

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

/// A packed spawn opens as one handle described by its path and header, and each chunk reads back as it was written; a
/// spawn that cannot be read leaves the one open as it was and no handle bound behind it.
#[test]
fn a_packed_spawn_opens_described_and_a_failed_open_leaves_no_binding() {
  let directory: PathBuf = build_absolute_generated_test_resource_path("spawn/session/loading");
  let (packed, corrupt): (PathBuf, PathBuf) = (directory.join("all.spawn"), directory.join("corrupt.spawn"));
  let source: SpawnFile = synthetic_spawn();

  fs::create_dir_all(&directory).unwrap();
  source.write_to_path::<XRayByteOrder, _>(&packed).unwrap();
  fs::write(&corrupt, b"invalid spawn").unwrap();

  let scopes: SessionScopes = SessionScopes::default();
  let files: SpawnFiles = SpawnFiles::new("spawn file", &scopes);
  let tool: SessionScope = SessionScope::new("main", "spawn-editor");
  let execution: ExecutionState = ExecutionState::new(ExecutionRequest::Auto).unwrap();
  let bind = || {
    let id: SessionId = SessionId::generate();

    scopes.bind(id, tool.clone());

    id
  };

  tauri::async_runtime::block_on(async {
    let opened = files
      .open_on_pool(&execution, bind(), "open", {
        let packed: PathBuf = packed.clone();

        move || SpawnSession::read(packed)
      })
      .await
      .unwrap();

    assert_eq!(opened.describe().path, packed);
    assert_eq!(opened.describe().header.guid, source.header.guid);
    assert_eq!(
      serde_json::to_value(match opened.read_chunk(SpawnChunkKind::Patrols) {
        SpawnChunk::Patrols(patrols) => patrols,
        other => panic!("a patrols chunk, not {other:?}"),
      })
      .unwrap(),
      serde_json::to_value(&source.patrols).unwrap()
    );

    let failed: SessionId = bind();

    assert!(
      files
        .open_on_pool(&execution, failed, "open", move || SpawnSession::read(corrupt))
        .await
        .is_err()
    );
    assert!(files.require(opened.session_id).is_ok());
    assert_eq!(
      scopes.list(&tool),
      vec![opened.session_id],
      "a failed open leaves no binding behind"
    );
  });
}

#[test]
fn a_chunk_is_tagged_with_its_kind() {
  let session: SpawnSession = SpawnSession::new(PathBuf::from("all.spawn"), synthetic_spawn());

  assert_eq!(
    serde_json::to_value(session.read_chunk(SpawnChunkKind::AlifeSpawn)).unwrap(),
    json!({ "kind": "alifeSpawn", "objects": [] })
  );
}
