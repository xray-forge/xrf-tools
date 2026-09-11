use std::sync::Arc;

use tauri::State;
use xrf_dltx::select_ltx_dialect;
use xrf_error::XrfResult;
use xrf_ltx::{LtxDocumentSource, LtxProject, LtxProjectOptions};
use xrf_ltx_inspect::{LtxInventory, LtxInventoryReader};
use xrf_utils::format_path;
use xrf_vfs::XrayRoots;

use crate::core::execution::ExecutionState;
use crate::core::types::TauriResult;
use crate::plugins::configs::descriptor::ConfigsProjectDescriptor;
use crate::plugins::configs::ltx_roots::open_ltx_project;
use crate::plugins::configs::request::ConfigsOpenRequest;
use crate::plugins::configs::state::{ConfigsProject, ConfigsState};

/// What opening one configs project produced, on its way back from the blocking thread.
///
/// The roots and prefix travel out and back rather than being cloned for the descriptor: the open owns them while it
/// runs, and the descriptor has to report the ones it actually mounted.
struct OpenedConfigsProject {
  project: LtxProject,
  inventory: LtxInventory,
  roots: XrayRoots,
  prefix: Option<String>,
}

/// Opens a configs project for browsing, and lists what it holds.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_project"))]
#[tauri::command(rename = "open_project")]
pub async fn configs_open_project(
  execution: State<'_, ExecutionState>,
  state: State<'_, ConfigsState>,
  request: ConfigsOpenRequest,
) -> TauriResult<Arc<ConfigsProjectDescriptor>> {
  let ConfigsOpenRequest {
    session_id,
    roots,
    prefix,
    is_dltx,
  } = request;

  log::info!("Opening ltx configs project in {}", roots.describe());

  // Claimed before the work, so a second open superseding this one cannot be committed over by it.
  state.begin_open(session_id)?;
  let state: ConfigsState = state.inner().clone();

  // Off the async worker: opening mounts every root, indexes the tree and reads the include list of every config in
  // it, which on an installation is thousands of files.
  let opened: OpenedConfigsProject = execution
    .run_blocking("Configs project open", move || open_and_list(roots, prefix, is_dltx))
    .await?
    .map_err(|error| error.to_string())?;

  let descriptor: Arc<ConfigsProjectDescriptor> = Arc::new(ConfigsProjectDescriptor {
    declared_schemes: opened
      .project
      .ltx_scheme_declarations
      .keys()
      .map(String::from)
      .collect::<Vec<String>>(),
    inventory: opened.inventory,
    is_dltx,
    prefix: opened.prefix,
    root: format_path(&opened.project.root).to_string(),
    roots: opened.roots,
    session_id,
  });

  // Shared rather than copied: the inventory of an installation is thousands of entries, and the session and every
  // later restore answer with the same one.
  state.commit_open(session_id, ConfigsProject::new(opened.project, Arc::clone(&descriptor)))?;

  Ok(descriptor)
}

/// Mounts the roots, assembles the project, and reads what it holds.
///
/// # Errors
///
/// Returns an error when the roots cannot be mounted, the project cannot be assembled, or a config it holds cannot be
/// read.
fn open_and_list(roots: XrayRoots, prefix: Option<String>, is_dltx: bool) -> XrfResult<OpenedConfigsProject> {
  let project: LtxProject = open_ltx_project(
    &roots,
    prefix.as_deref(),
    LtxProjectOptions {
      dialect: select_ltx_dialect(is_dltx),
      is_with_schemes_check: true,
      is_strict_check: false,
    },
  )?;

  // Scoped, so the borrow the reader holds ends before the project moves out with the answer.
  let inventory: LtxInventory = {
    let source = project.document_source();

    LtxInventoryReader::new(&project, &source as &dyn LtxDocumentSource).read()?
  };

  Ok(OpenedConfigsProject {
    inventory,
    prefix,
    project,
    roots,
  })
}
