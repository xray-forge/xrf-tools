//! Pins what `open_project` does with a scripts project: the externs a readable one answers, and the handle a missing
//! one leaves behind, which is none.

use std::fs;
use std::path::{Path, PathBuf};

use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};
use tauri::{App, Manager};
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionScope, SessionScopes};
use xrf_export::ExportsProject;
use xrf_job::ExecutionRequest;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::ExportsProjects;
use crate::commands::open_project::exports_open_project;
use crate::plugin::ExportsPlugin;

/// The application as far as the plugin needs it, set up by the plugin itself.
fn application() -> App<MockRuntime> {
  mock_builder()
    .manage(SessionScopes::default())
    .manage(ExecutionState::new(ExecutionRequest::Auto).expect("pool starts"))
    .plugin(ExportsPlugin::init())
    .build(mock_context(noop_assets()))
    .expect("the plugin sets up")
}

/// Opens `root` as the exports tool would, under a handle bound to the tool's scope.
fn open(application: &App<MockRuntime>, tool: &SessionScope, root: &Path) -> (SessionId, Result<(), String>) {
  let session_id: SessionId = SessionId::generate();

  application.state::<SessionScopes>().bind(session_id, tool.clone());

  let opened = tauri::async_runtime::block_on(exports_open_project(
    session_id,
    root.display().to_string(),
    application.state(),
    application.state(),
  ));

  (session_id, opened.map(|_| ()))
}

#[test]
fn a_project_opens_with_every_extern_it_declares() {
  let application: App<MockRuntime> = application();
  let tool: SessionScope = SessionScope::new("main", "exports-explorer");
  let root: PathBuf = build_absolute_generated_test_resource_path("exports/open/declared");
  let source: PathBuf = root.join("src/engine/tasks.ts");

  let _ = fs::remove_dir_all(&root);

  fs::create_dir_all(source.parent().expect("a file sits in a directory")).expect("test tree is creatable");
  fs::write(&source, r#"extern("tasks.run", (count?: number): boolean => true);"#).expect("test file is writable");

  let (session_id, opened) = open(&application, &tool, &root);

  opened.expect("the project opens");

  let project = application
    .state::<ExportsProjects>()
    .require(session_id)
    .expect("the handle holds the project");
  let project: &ExportsProject = &project;

  assert_eq!(project.root, root);
  assert_eq!(
    project
      .declarations
      .iter()
      .map(|declaration| declaration.name.as_str())
      .collect::<Vec<&str>>(),
    vec!["tasks.run"]
  );
}

#[test]
fn a_missing_project_opens_nothing_and_leaves_no_binding() {
  let application: App<MockRuntime> = application();
  let tool: SessionScope = SessionScope::new("main", "exports-explorer");
  let root: PathBuf = build_absolute_generated_test_resource_path("exports/open/missing");

  let _ = fs::remove_dir_all(&root);

  let (session_id, opened) = open(&application, &tool, &root);

  assert!(opened.is_err(), "a missing project is refused");
  assert!(application.state::<ExportsProjects>().require(session_id).is_err());
  assert!(
    application.state::<SessionScopes>().list(&tool).is_empty(),
    "a failed open leaves no binding behind"
  );
}
