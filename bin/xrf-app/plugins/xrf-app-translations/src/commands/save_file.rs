use std::collections::HashMap;

use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionId;
use xrf_translation::TranslationEdit;

use crate::{TranslationProjects, TranslationSaveOutcome, TranslationSavePlan};

/// Write one logical file's pending edits, grouped by the language each belongs to, into the project `session_id` holds.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "save_file")]
pub async fn translations_save_file(
  session_id: SessionId,
  file: String,
  edits: HashMap<String, Vec<TranslationEdit>>,
  projects: State<'_, TranslationProjects>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<TranslationSaveOutcome> {
  let (projects, assets): (TranslationProjects, AssetMountState) = (projects.inner().clone(), assets.inner().clone());

  // Off the async runtime: the commit syncs every file to disk and moves every open session before it answers.
  execution
    .run_blocking("Saving translations", move || {
      TranslationSavePlan::save(&projects, &assets, session_id, &file, &edits)
    })
    .await?
}
