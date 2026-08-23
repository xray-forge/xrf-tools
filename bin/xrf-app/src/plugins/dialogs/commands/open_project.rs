use tauri::State;
use xrf_dialog::{DialogProject, DialogProjectDescriptor, DialogProjectMode, DialogProjectOptions};
use xrf_vfs::XrayMountMode;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;
use crate::plugins::dialogs::state::DialogProjectState;

/// Open a dialog tree, in the layout the caller names.
///
/// The layout mode is obeyed, never re-derived: it decides which files a later save writes, so a guess
/// acted on here would decide what gets overwritten. `detect_mode` is what preselects it.
///
/// Both prefix overrides are optional and each stands in for one logical prefix, so a mod keeping its
/// dialogs somewhere the layout does not predict still opens.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_project"))]
#[tauri::command(rename = "open_project")]
pub async fn dialogs_open_project(
  path: &str,
  source: XrayMountMode,
  mode: DialogProjectMode,
  dialogs_prefix: Option<String>,
  translations_prefix: Option<String>,
  state: State<'_, DialogProjectState>,
) -> TauriResult<DialogProjectDescriptor> {
  log::info!("Opening dialogs project: {} ({:?}, {:?})", path, source, mode);

  let options: DialogProjectOptions = DialogProjectOptions {
    dialogs_prefix,
    translations_prefix,
    ..DialogProjectOptions::new(path, mode)
  };

  let project: DialogProject = DialogProject::open_with_mode(source, &options).map_err(error_to_string)?;
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
