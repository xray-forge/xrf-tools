use std::collections::{HashMap, HashSet};

use tauri::State;
use xrf_translation::{TranslationEdit, build_save_transaction};
use xrf_utils::error_to_string;
use xrf_vfs::{XrayChangeSet, XrayInputKind, XrayInputRecord, XrayLogicalPath, XraySearch};

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

/// Write `edits` into whichever project is open, and answer the project as the commit left it while it still is.
pub(in crate::plugins::translations) fn save_into_open_project(
  state: &TranslationProjectState,
  assets: &AssetMountState,
  session_id: SessionId,
  file: &str,
  edits: &HashMap<String, Vec<TranslationEdit>>,
) -> TauriResult<TranslationSaveOutcome> {
  let plan: TranslationSavePlan = state.begin_save(session_id, file)?;
  let changes: XrayChangeSet = write_edits(assets, &plan, edits)?;

  state.settle_save(&plan, &changes)
}

/// Commit `edits` to the files `plan` names as one transaction, touching no translations state: the commit moves every
/// open session along, this project's among them, which reads itself again.
///
/// Refused, with nothing written, when a file it writes changed since the plan's project read it, so a save never
/// overwrites what it did not see; a change to any other file of the project does not stand in its way.
pub(in crate::plugins::translations) fn write_edits(
  assets: &AssetMountState,
  plan: &TranslationSavePlan,
  edits: &HashMap<String, Vec<TranslationEdit>>,
) -> TauriResult<XrayChangeSet> {
  let sources: XraySearch = assets
    .search(&plan.roots)?
    .with_optional_prefix(Some(&plan.prefix))
    .map_err(error_to_string)?;
  let written: HashSet<XrayLogicalPath> = plan
    .sources
    .values()
    .filter_map(|source| XrayLogicalPath::new(&source.logical_path).ok())
    .collect();
  let expected: XrayInputRecord = plan
    .project
    .inputs
    .filtered(|input| matches!(input.get_kind(), XrayInputKind::File(path) if written.contains(path)));

  assets.commit(
    &build_save_transaction(&sources, &plan.file, &plan.sources, edits)
      .map_err(error_to_string)?
      .with_expected(&expected),
  )
}
