use std::collections::HashMap;

use tauri::State;
use xrf_translation::{
  TranslationEdit, TranslationProjectDescriptor, TranslationProjectMode, build_save_transaction, read_gamedata_in,
  read_source_in,
};
use xrf_utils::error_to_string;
use xrf_vfs::XraySearch;

use crate::core::assets::AssetMountState;
use crate::core::session::SessionId;
use crate::core::types::TauriResult;
use crate::plugins::translations::state::{TranslationProjectState, TranslationSaveOutcome, TranslationSavePlan};

/// Write one logical file's pending edits, grouped by the language each belongs to.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "save_file"))]
#[tauri::command(rename = "save_file")]
pub async fn translations_save_file(
  session_id: SessionId,
  file: &str,
  edits: HashMap<String, Vec<TranslationEdit>>,
  state: State<'_, TranslationProjectState>,
  assets: State<'_, AssetMountState>,
) -> TauriResult<TranslationSaveOutcome> {
  save_into_open_project(&state, &assets, session_id, file, &edits)
}

/// Write `edits` into whichever project is open, and adopt what they left on disk while it still is.
pub(in crate::plugins::translations) fn save_into_open_project(
  state: &TranslationProjectState,
  assets: &AssetMountState,
  session_id: SessionId,
  file: &str,
  edits: &HashMap<String, Vec<TranslationEdit>>,
) -> TauriResult<TranslationSaveOutcome> {
  let plan: TranslationSavePlan = state.begin_save(session_id, file)?;
  let refreshed: TranslationProjectDescriptor = write_edits(assets, &plan, edits)?;

  state.commit_save(&plan, refreshed)
}

/// Commit `edits` to the files `plan` names as one transaction and read the project back from the snapshot it leaves,
/// touching no translations state at all.
pub(in crate::plugins::translations) fn write_edits(
  assets: &AssetMountState,
  plan: &TranslationSavePlan,
  edits: &HashMap<String, Vec<TranslationEdit>>,
) -> TauriResult<TranslationProjectDescriptor> {
  let sources: XraySearch = assets
    .search(&plan.roots)?
    .with_optional_prefix(Some(&plan.prefix))
    .map_err(error_to_string)?;

  assets.commit(&build_save_transaction(&sources, &plan.file, &plan.sources, edits).map_err(error_to_string)?)?;

  let search: XraySearch = assets.search(&plan.roots)?;

  match plan.mode {
    TranslationProjectMode::Source => read_source_in(&search, &plan.roots, &plan.prefix),
    TranslationProjectMode::Gamedata => read_gamedata_in(&search, &plan.roots, &plan.prefix),
  }
  .map_err(error_to_string)
}
