use std::sync::Arc;

use tauri::State;
use xrf_translation::TranslationProjectDescriptor;

use crate::core::assets::AssetMountState;
use crate::core::session::{SessionDerived, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::translations::project_read::read_project;
use crate::plugins::translations::request::TranslationsOpenRequest;
use crate::plugins::translations::state::TranslationProjectState;

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

  state.session.begin_open(session_id)?;

  let prefix: String = prefix.unwrap_or_else(|| mode.get_prefix().to_owned());

  log::info!(
    "Opening translations project: {} root(s), {:?}, '{}'",
    roots.roots.len(),
    mode,
    prefix
  );

  // Through the shared roots, recording what it reads, so a commit to any of those files reads the project again.
  let read: SessionDerived<TranslationProjectDescriptor> =
    read_project(assets.search_fresh(&roots)?, &roots, mode, &prefix)?;

  log::info!(
    "Opened {} translation files, {} languages, {} findings, editable: {}",
    read.value.files.len(),
    read.value.languages.len(),
    read.value.findings.len(),
    read.value.is_editable
  );

  // Committed only once the read succeeded, so a failed open leaves whatever was already open in place.
  state
    .session
    .commit_open_with(session_id, read.value, read.inputs, read.parts)
}
