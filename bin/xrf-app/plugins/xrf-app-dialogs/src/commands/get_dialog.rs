use tauri::State;
use xrf_app_core::TauriResult;
use xrf_dialog::{DialogDescriptor, DialogProject};

use crate::DialogProjects;
use crate::request::DialogsReadRequest;

/// One dialog, with every phrase it declares.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "get_dialog")]
pub async fn dialogs_get_dialog(
  request: DialogsReadRequest,
  projects: State<'_, DialogProjects>,
) -> TauriResult<DialogDescriptor> {
  let DialogsReadRequest {
    session_id,
    logical_path,
    id,
    language,
  } = request;
  let opened = projects.require(session_id)?;
  let project: &DialogProject = &opened.project;

  if project.find_file(&logical_path).is_none() {
    return Err(format!("The open dialogs project holds no file '{logical_path}'"));
  }

  project
    .describe_dialog(&logical_path, &id, language.as_deref())
    .ok_or_else(|| format!("No dialog '{id}' in '{logical_path}'"))
}
