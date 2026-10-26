//! Pins a subject's life through the browse commands: a world opened over its roots, read however a reference spells an
//! entry, restored for the tool that opened it, and closed with nothing left bound; and a volume set read and
//! extracted under the same spellings a world takes.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};
use tauri::{App, Manager};
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionScope, SessionScopes, SessionSnapshot};
use xrf_job::ExecutionRequest;
use xrf_test_utils::GeneratedResources;
use xrf_vfs::{XrayMountMode, XrayRoot, XrayRoots};

use crate::browse::commands::close_subject::archives_close_subject;
use crate::browse::commands::extract_file::archives_extract_file;
use crate::browse::commands::open_volumes::archives_open_volumes;
use crate::browse::commands::open_world::archives_open_world;
use crate::browse::commands::read_file::archives_read_file;
use crate::browse::{ArchiveSubject, ArchiveSubjects};
use crate::plugin::ArchivesPlugin;
use crate::tests::fixtures;

/// The application as far as the plugin needs it, set up by the plugin itself.
fn application() -> App<MockRuntime> {
  mock_builder()
    .manage(SessionScopes::default())
    .manage(AssetMountState::new())
    .manage(ExecutionState::new(ExecutionRequest::Auto).expect("pool starts"))
    .plugin(ArchivesPlugin::init())
    .build(mock_context(noop_assets()))
    .expect("the plugin sets up")
}

#[test]
fn a_world_opens_over_its_roots_reads_restores_and_closes() {
  let application: App<MockRuntime> = application();
  let tool: SessionScope = SessionScope::new("main", "archives-explorer");
  let root: PathBuf = fixtures::tree("browse/cycle", &[("configs/system.ltx", b"[system]")]);
  let roots: XrayRoots = XrayRoots::new([XrayRoot::new(root, XrayMountMode::Directory)]);
  let session_id: SessionId = SessionId::generate();

  application.state::<SessionScopes>().bind(session_id, tool.clone());

  let opened: Arc<SessionSnapshot<ArchiveSubject>> = tauri::async_runtime::block_on(archives_open_world(
    session_id,
    roots.clone(),
    application.state(),
    application.state(),
    application.state(),
  ))
  .expect("the world opens");

  let ArchiveSubject::World {
    world,
    roots: opened_over,
  } = &**opened
  else {
    panic!("a game folder opens as a world");
  };

  assert_eq!(opened_over, &roots, "a world answers the roots it was opened over");
  assert_eq!(
    world
      .files
      .iter()
      .map(|entry| entry.name.as_str())
      .collect::<Vec<&str>>(),
    vec!["configs\\system.ltx"]
  );

  // A reference spelled in another case and with forward slashes names the same engine path.
  let read = tauri::async_runtime::block_on(archives_read_file(
    session_id,
    String::from("Configs/System.LTX"),
    application.state(),
    application.state(),
  ))
  .expect("the entry reads");

  assert_eq!(read.content, "[system]");
  assert_eq!((read.session_id, read.revision), (opened.session_id, opened.revision));

  let subjects = application.state::<ArchiveSubjects>();

  assert!(Arc::ptr_eq(
    &subjects.restore(&tool).expect("the tool's subject restores"),
    &opened
  ));

  archives_close_subject(application.state(), vec![session_id]);

  assert!(subjects.require(session_id).is_err());
  assert!(subjects.restore(&tool).is_none());
  assert!(
    application.state::<SessionScopes>().list(&tool).is_empty(),
    "a closed subject leaves no binding behind"
  );
}

#[test]
fn a_volume_set_reads_and_extracts_an_entry_however_a_reference_spells_it() {
  let application: App<MockRuntime> = application();
  let tool: SessionScope = SessionScope::new("main", "archives-explorer");
  let volumes: PathBuf = fixtures::volumes("browse/volumes", &[("configs/system.ltx", b"[system]")]);
  let extracted: PathBuf = GeneratedResources::path("archives/browse/volumes/extracted/system.ltx");
  let session_id: SessionId = SessionId::generate();

  let _ = fs::remove_file(&extracted);

  application.state::<SessionScopes>().bind(session_id, tool);

  tauri::async_runtime::block_on(archives_open_volumes(
    session_id,
    volumes,
    application.state(),
    application.state(),
  ))
  .expect("the volumes open");

  // The name table lists the entry as authored; a reference in another case and with forward slashes is the same file,
  // as it is in a world.
  let read = tauri::async_runtime::block_on(archives_read_file(
    session_id,
    String::from("Configs/System.LTX"),
    application.state(),
    application.state(),
  ))
  .expect("the entry reads");

  assert_eq!(read.content, "[system]");

  tauri::async_runtime::block_on(archives_extract_file(
    session_id,
    String::from("Configs/System.LTX"),
    extracted.clone(),
    application.state(),
    application.state(),
  ))
  .expect("the entry extracts");

  assert_eq!(
    fs::read_to_string(&extracted).expect("the extracted file reads"),
    "[system]"
  );

  archives_close_subject(application.state(), vec![session_id]);
}
