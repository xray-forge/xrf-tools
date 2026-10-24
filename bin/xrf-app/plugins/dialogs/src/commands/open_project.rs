use tauri::State;
use xrf_app_core::session::SessionSnapshot;
use xrf_app_core::types::TauriResult;
use xrf_dialog::{DialogProject, DialogProjectDescriptor, DialogProjectLayout};
use xrf_utils::error_to_string;

use crate::request::DialogsOpenRequest;
use crate::state::DialogProjectState;

/// Open a dialog tree.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_project"))]
#[tauri::command(rename = "open_project")]
pub async fn dialogs_open_project(
  request: DialogsOpenRequest,
  state: State<'_, DialogProjectState>,
) -> TauriResult<SessionSnapshot<DialogProjectDescriptor>> {
  let DialogsOpenRequest {
    session_id,
    roots,
    mode,
    dialogs_prefix,
    translations_prefix,
  } = request;

  state.begin_open(session_id)?;

  log::info!("Opening dialogs project: {} root(s), {:?}", roots.roots.len(), mode);

  let layout: DialogProjectLayout = DialogProjectLayout {
    dialogs_prefix,
    translations_prefix,
    ..DialogProjectLayout::new(mode)
  };

  let project: DialogProject = DialogProject::open(&roots, &layout).map_err(error_to_string)?;
  let descriptor: DialogProjectDescriptor = project.describe();

  log::info!(
    "Opened {} dialog files, {} dialogs, {} findings, editable: {}",
    descriptor.files.len(),
    project.sum_dialogs(),
    descriptor.findings.len(),
    descriptor.is_editable
  );

  Ok(state.commit_open(session_id, project)?.map(|_| descriptor))
}
