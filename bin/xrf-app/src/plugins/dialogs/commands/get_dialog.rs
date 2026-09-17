use tauri::State;
use xrf_dialog::DialogDescriptor;

use crate::core::types::TauriResult;
use crate::plugins::dialogs::request::DialogsReadRequest;
use crate::plugins::dialogs::state::DialogProjectState;

/// One dialog, with every phrase it declares.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_dialog"))]
#[tauri::command(rename = "get_dialog")]
pub async fn dialogs_get_dialog(
  request: DialogsReadRequest,
  state: State<'_, DialogProjectState>,
) -> TauriResult<DialogDescriptor> {
  let DialogsReadRequest {
    session_id,
    logical_path,
    id,
    language,
  } = request;
  let project = state.require(session_id)?;

  if project.find_file(&logical_path).is_none() {
    return Err(format!("The open dialogs project holds no file '{logical_path}'"));
  }

  project
    .describe_dialog(&logical_path, &id, language.as_deref())
    .ok_or_else(|| format!("No dialog '{id}' in '{logical_path}'"))
}
