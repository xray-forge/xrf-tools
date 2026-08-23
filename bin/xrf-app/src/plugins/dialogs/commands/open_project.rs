use std::path::{Path, PathBuf};

use tauri::State;
use xrf_dialog::{DialogProject, DialogProjectDescriptor, DialogProjectMode, DialogProjectOverrides};

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;
use crate::plugins::dialogs::state::DialogProjectState;

/// Open a dialog tree, in the layout the caller names.
///
/// The mode is obeyed, never re-derived: it decides which files a later save writes, so a guess
/// acted on here would decide what gets overwritten. `detect_mode` is what preselects it.
///
/// Both path overrides are optional and each stands in for one root. Source mode needs them least
/// often and gamedata mode most: a mod that keeps its dialogs somewhere the layout does not predict
/// is otherwise unopenable.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_project"))]
#[tauri::command(rename = "open_project")]
pub async fn dialogs_open_project(
  path: &str,
  mode: DialogProjectMode,
  dialogs_path: Option<String>,
  translations_path: Option<String>,
  state: State<'_, DialogProjectState>,
) -> TauriResult<DialogProjectDescriptor> {
  log::info!("Opening dialogs project: {} ({:?})", path, mode);

  let overrides: DialogProjectOverrides = DialogProjectOverrides {
    dialogs: dialogs_path.map(PathBuf::from),
    translations: translations_path.map(PathBuf::from),
  };

  let project: DialogProject = DialogProject::open(Path::new(path), mode, &overrides).map_err(error_to_string)?;
  let descriptor: DialogProjectDescriptor = project.describe();

  log::info!(
    "Opened {} dialog files, {} dialogs, {} findings",
    descriptor.files.len(),
    project.sum_dialogs(),
    descriptor.findings.len()
  );

  *state.project.lock().unwrap() = Some(project);

  Ok(descriptor)
}
