use std::sync::MutexGuard;

use tauri::State;
use xrf_dialog::{DialogDescriptor, DialogProject};

use crate::core::types::TauriResult;
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
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_dialog"))]
#[tauri::command(rename = "get_dialog")]
pub async fn dialogs_get_dialog(
  logical_path: String,
  id: String,
  state: State<'_, DialogProjectState>,
) -> TauriResult<DialogDescriptor> {
  let lock: MutexGuard<Option<DialogProject>> = state.project.lock().unwrap();
  let Some(project) = lock.as_ref() else {
    return Err(String::from("No dialogs project is open"));
  };

  if project.find_file(&logical_path).is_none() {
    return Err(format!("The open dialogs project holds no file '{logical_path}'"));
  }

  project
    .describe_dialog(&logical_path, &id)
    .ok_or_else(|| format!("No dialog '{id}' in '{logical_path}'"))
}
