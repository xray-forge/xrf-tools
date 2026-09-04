use tauri::State;
use xrf_dialog::{DialogProject, DialogProjectDescriptor, DialogProjectLayout};

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;
use crate::plugins::dialogs::request::DialogsOpenRequest;
use crate::plugins::dialogs::state::DialogProjectState;

/// Open a dialog tree.
///
/// Two arguments, because opening answers two questions. `roots` is the shared vocabulary every
/// surface names roots with — ordered roots, each with its own mount mode — so an installation opens
/// as readily as a loose tree and a gamedata tree layers in front of one. `layout` is this domain's
/// own half: where inside those trees the dialogs and their text sit.
///
/// The layout mode is obeyed, never re-derived: it decides which files a later save writes, so a guess
/// acted on here would decide what gets overwritten. `detect_mode` is what preselects it.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_project"))]
#[tauri::command(rename = "open_project")]
pub async fn dialogs_open_project(
  request: DialogsOpenRequest,
  state: State<'_, DialogProjectState>,
) -> TauriResult<DialogProjectDescriptor> {
  let DialogsOpenRequest {
    roots,
    mode,
    dialogs_prefix,
    translations_prefix,
  } = request;

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

  *state.project.lock().unwrap() = Some(project);

  Ok(descriptor)
}
