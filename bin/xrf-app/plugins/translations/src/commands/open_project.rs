use std::sync::Arc;

use tauri::State;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionDerived, SessionSnapshot};
use xrf_app_core::types::TauriResult;
use xrf_translation::TranslationProjectDescriptor;
use xrf_vfs::{XrayRevision, XraySearch};

use crate::request::TranslationsOpenRequest;
use crate::state::TranslationProjectState;

/// Open a translations tree.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_project"))]
#[tauri::command(rename = "open_project")]
pub async fn translations_open_project(
  request: TranslationsOpenRequest,
  state: State<'_, TranslationProjectState>,
  assets: State<'_, AssetMountState>,
) -> TauriResult<Arc<SessionSnapshot<TranslationProjectDescriptor>>> {
  let TranslationsOpenRequest {
    session_id,
    roots,
    mode,
    prefix,
  } = request;

  state.projects.begin_open(session_id)?;

  let prefix: String = prefix.unwrap_or_else(|| mode.get_prefix().to_owned());

  log::info!(
    "Opening translations project: {} root(s), {:?}, '{}'",
    roots.roots.len(),
    mode,
    prefix
  );

  // Through the shared roots, recording what it reads, so a commit to any of those files reads the project again.
  let search: XraySearch = assets.search_fresh(&roots)?;
  let revision: XrayRevision = search.get_vfs().get_revision();
  let read: SessionDerived<TranslationProjectDescriptor> =
    TranslationProjectState::read_project(search, &roots, mode, &prefix)?;

  log::info!(
    "Opened {} translation files, {} languages, {} findings, editable: {}",
    read.value.files.len(),
    read.value.languages.len(),
    read.value.findings.len(),
    read.value.is_editable
  );

  // Committed only once the read succeeded, so a failed open leaves whatever was already open in place.
  let opened: Arc<SessionSnapshot<TranslationProjectDescriptor>> =
    state
      .projects
      .commit_open_with(session_id, read.value, revision, read.inputs, read.parts)?;

  Ok(state.projects.catch_up(opened, &assets))
}
