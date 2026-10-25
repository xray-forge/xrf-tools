//! Pins the handle `open_model` leaves behind when the visual cannot be read, which is none.
//!
//! What a readable visual is described as is `xrf-visual`'s to pin, over the visuals its own fixtures build.

use std::fs;
use std::path::PathBuf;

use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};
use tauri::{App, Manager};
use xrf_app_core::assets::{AssetMountState, AssetSource};
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionScope, SessionScopes};
use xrf_job::ExecutionRequest;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::XrayRoots;

use crate::VisualModels;
use crate::commands::open_model::visuals_open_model;
use crate::plugin::VisualsPlugin;

/// The application as far as the plugin needs it, set up by the plugin itself.
fn application() -> App<MockRuntime> {
  mock_builder()
    .manage(SessionScopes::default())
    .manage(AssetMountState::new())
    .manage(ExecutionState::new(ExecutionRequest::Auto).expect("pool starts"))
    .plugin(VisualsPlugin::init())
    .build(mock_context(noop_assets()))
    .expect("the plugin sets up")
}

#[test]
fn a_visual_that_cannot_be_read_opens_nothing_and_leaves_no_binding() {
  let application: App<MockRuntime> = application();
  let tool: SessionScope = SessionScope::new("main", "visuals-explorer");
  let directory: PathBuf = build_absolute_generated_test_resource_path("visuals/open/unreadable");
  let path: PathBuf = directory.join("stalker.ogf");
  let session_id: SessionId = SessionId::generate();

  let _ = fs::remove_dir_all(&directory);

  fs::create_dir_all(&directory).expect("test directory is creatable");
  fs::write(&path, b"not a visual").expect("test file is writable");
  application.state::<SessionScopes>().bind(session_id, tool.clone());

  let opened = tauri::async_runtime::block_on(visuals_open_model(
    session_id,
    AssetSource::File {
      path: path.display().to_string(),
    },
    XrayRoots::default(),
    application.state(),
    application.state(),
    application.state(),
  ));

  assert!(opened.is_err(), "an unreadable visual is refused");
  assert!(application.state::<VisualModels>().require(session_id).is_err());
  assert!(
    application.state::<SessionScopes>().list(&tool).is_empty(),
    "a failed open leaves no binding behind"
  );
}
