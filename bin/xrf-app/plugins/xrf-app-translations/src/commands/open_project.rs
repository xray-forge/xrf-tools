use std::sync::Arc;

use tauri::State;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionDerived, SessionOpening, SessionSnapshot};
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

  // Given up as the open fails, so a failed open leaves the tool's other openings as they were.
  let opening: SessionOpening<TranslationProjectDescriptor> = state.projects.begin_open(session_id)?;

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

  let opened: Arc<SessionSnapshot<TranslationProjectDescriptor>> = opening.commit_derived(read, revision)?;

  Ok(state.projects.catch_up(opened, &assets))
}
