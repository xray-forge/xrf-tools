//! Pins what `open_project` does with a tree: the description a readable one answers, and the handle a tree with no
//! dialogs leaves behind, which is none.

use std::fs;
use std::path::{Path, PathBuf};

use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};
use tauri::{App, Manager};
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionScope, SessionScopes};
use xrf_dialog::DialogProjectMode;
use xrf_job::ExecutionRequest;
use xrf_test_utils::GeneratedResources;
use xrf_vfs::{XrayMountMode, XrayRoot, XrayRoots};

use crate::DialogProjects;
use crate::commands::open_project::dialogs_open_project;
use crate::plugin::DialogsPlugin;
use crate::request::DialogsOpenRequest;

const DIALOG: &str = r#"<game_dialogs>
  <dialog id="escape_trader">
    <phrase_list>
      <phrase id="0"><text>escape_trader_0</text></phrase>
    </phrase_list>
  </dialog>
</game_dialogs>"#;

/// The application as far as the plugin needs it, set up by the plugin itself.
fn application() -> App<MockRuntime> {
  mock_builder()
    .manage(SessionScopes::default())
    .manage(AssetMountState::new())
    .manage(ExecutionState::new(ExecutionRequest::Auto).expect("pool starts"))
    .plugin(DialogsPlugin::init())
    .build(mock_context(noop_assets()))
    .expect("the plugin sets up")
}

/// An empty tree of one case's own, with `files` written into it.
fn tree(case: &str, files: &[(&str, &str)]) -> PathBuf {
  let root: PathBuf = GeneratedResources::path(&format!("dialogs/open/{case}"));

  let _ = fs::remove_dir_all(&root);

  fs::create_dir_all(&root).expect("test tree is creatable");

  for (path, contents) in files {
    let path: PathBuf = root.join(path);

    fs::create_dir_all(path.parent().expect("a file sits in a directory")).expect("test tree is creatable");
    fs::write(&path, contents).expect("test file is writable");
  }

  root
}

/// Opens `root` as the dialogs tool would, under a handle bound to the tool's scope.
fn open(application: &App<MockRuntime>, tool: &SessionScope, root: &Path) -> (SessionId, Result<(), String>) {
  let session_id: SessionId = SessionId::generate();

  application.state::<SessionScopes>().bind(session_id, tool.clone());

  let opened = tauri::async_runtime::block_on(dialogs_open_project(
    DialogsOpenRequest {
      session_id,
      roots: XrayRoots::new([XrayRoot::new(root.to_path_buf(), XrayMountMode::Directory)]),
      mode: DialogProjectMode::Gamedata,
      dialogs_prefix: None,
      translations_prefix: None,
    },
    application.state(),
    application.state(),
    application.state(),
  ));

  (session_id, opened.map(|_| ()))
}

#[test]
fn a_tree_opens_described_by_its_dialog_files() {
  let application: App<MockRuntime> = application();
  let tool: SessionScope = SessionScope::new("main", "dialogs-editor");
  let root: PathBuf = tree("described", &[("configs/gameplay/dialogs_escape.xml", DIALOG)]);
  let (session_id, opened) = open(&application, &tool, &root);

  opened.expect("the tree opens");

  let described = application
    .state::<DialogProjects>()
    .require(session_id)
    .expect("the handle holds the project")
    .describe();
  let file = described
    .files
    .get(r"configs\gameplay\dialogs_escape.xml")
    .expect("the file is described by its logical path");

  assert_eq!(described.dialogs_prefix, r"configs\gameplay");
  assert_eq!(described.files.len(), 1);
  assert!(file.is_editable, "a loose file can be written back");
  assert_eq!(
    file
      .dialogs
      .iter()
      .map(|dialog| dialog.id.as_str())
      .collect::<Vec<&str>>(),
    vec!["escape_trader"]
  );
}

#[test]
fn a_tree_holding_no_dialogs_opens_nothing_and_leaves_no_binding() {
  let application: App<MockRuntime> = application();
  let tool: SessionScope = SessionScope::new("main", "dialogs-editor");
  let (session_id, opened) = open(&application, &tool, &tree("empty", &[]));

  assert!(
    opened
      .expect_err("an empty tree is refused")
      .contains("No dialog files")
  );
  assert!(application.state::<DialogProjects>().require(session_id).is_err());
  assert!(
    application.state::<SessionScopes>().list(&tool).is_empty(),
    "a failed open leaves no binding behind"
  );
}
