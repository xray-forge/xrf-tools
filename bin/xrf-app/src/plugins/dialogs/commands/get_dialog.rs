use tauri::State;
use xrf_dialog::DialogDescriptor;

use crate::core::types::TauriResult;
use crate::plugins::dialogs::request::DialogsReadRequest;
use crate::plugins::dialogs::state::DialogProjectState;

/// One dialog, with every phrase it declares.
///
/// The project response carries only summaries — 502 dialogs' worth of phrases is a payload nobody
/// reads — so this is what a selection fetches. Served from the parsed project already in state, so
/// it costs a lookup rather than a read.
///
/// Addressed by file and id together, because ids are not unique across a tree: a mod overlaying a
/// dialog keeps the original's id, and searching every file would silently answer with whichever copy
/// was read first.
///
/// `language` picks which of the project's languages the phrase lines are resolved in, defaulting to
/// the first the text tree offers. Switching language is another call rather than a payload carrying
/// all of them: the index is already resident, so it costs a lookup, and one dialog in nine languages
/// would be nine times the bytes to display an eighth of it.
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
