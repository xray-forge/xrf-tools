use std::sync::Arc;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionSnapshot;
use xrf_translation::TranslationProjectDescriptor;

use crate::request::TranslationsOpenRequest;
use crate::{TranslationProjectRead, TranslationProjects};

/// Open a translations tree through the shared roots, read on the pool, recording what it reads so a commit to any of
/// those files reads the project again.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "open_project")]
pub async fn translations_open_project(
  request: TranslationsOpenRequest,
  projects: State<'_, TranslationProjects>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<Arc<SessionSnapshot<TranslationProjectDescriptor>>> {
  let TranslationsOpenRequest {
    session_id,
    roots,
    mode,
    prefix,
  } = request;
  let prefix: String = prefix.unwrap_or_else(|| mode.get_prefix().to_owned());

  log::info!(
    "Opening translations project: {} root(s), {:?}, '{}'",
    roots.roots.len(),
    mode,
    prefix
  );

  let read_roots = roots.clone();
  let opened: Arc<SessionSnapshot<TranslationProjectDescriptor>> = projects
    .open_followed(
      &execution,
      &assets,
      session_id,
      roots,
      "Opening a translations project",
      move |search| TranslationProjectRead::read(search, &read_roots, mode, &prefix),
    )
    .await?;

  log::info!(
    "Opened {} translation files, {} languages, {} findings, editable: {}",
    opened.files.len(),
    opened.languages.len(),
    opened.findings.len(),
    opened.is_editable
  );

  Ok(opened)
}
