use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionSnapshot;
use xrf_dialog::{DialogProject, DialogProjectDescriptor, DialogProjectLayout};
use xrf_utils::error_to_string;

use crate::request::DialogsOpenRequest;
use crate::{DialogProjects, OpenedDialogProject};

/// Open a dialog tree, reading every dialog file and string table it holds on the pool.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "open_project")]
pub async fn dialogs_open_project(
  request: DialogsOpenRequest,
  projects: State<'_, DialogProjects>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<Arc<DialogProjectDescriptor>>> {
  let DialogsOpenRequest {
    session_id,
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
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let opened: Arc<SessionSnapshot<OpenedDialogProject>> = projects
    .open_on_pool(&execution, session_id, "Opening a dialogs project", move || {
      // Through the shared mounts, read afresh, as every open a person makes is.
      let project: DialogProject =
        DialogProject::from_search(assets.search_fresh(&roots)?, &roots, &layout).map_err(error_to_string)?;

      Ok(OpenedDialogProject::new(project))
    })
    .await?;

  log::info!(
    "Opened {} dialog files, {} dialogs, {} findings, editable: {}",
    opened.descriptor.files.len(),
    opened.project.sum_dialogs(),
    opened.descriptor.findings.len(),
    opened.descriptor.is_editable
  );

  Ok(opened.map(OpenedDialogProject::describe))
}
