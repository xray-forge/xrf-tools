use std::collections::HashMap;

use tauri::State;
use xrf_translation::{
  TranslationEdit, TranslationFile, TranslationProjectDescriptor, TranslationProjectMode, TranslationSource,
  apply_edits_to_asset, read_gamedata_in, read_source_in,
};
use xrf_vfs::{XrayAsset, XrayLookupScope, XrayScopedVfs, XrayVfs};

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;
use crate::plugins::translations::state::TranslationProjectState;

/// Write one logical file's pending edits, grouped by the language each belongs to.
///
/// A logical file is several files on disk in gamedata mode, one per language, so the edits arrive
/// keyed by language and each group goes to its own source. The sources come from the open project
/// rather than from the caller, so a save can only ever touch files this project actually read.
///
/// Each one is re-resolved through the VFS before it is written. The descriptor's own path is portable
/// and therefore lossy — a display form — and using it as a write address is how an edit lands
/// somewhere that is not the file. The mount answers with the real path and with what wins *now*.
///
/// A language served out of an archive is refused by name rather than skipped, because a save that
/// silently drops one language's edits looks identical to one that succeeded.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "save_file"))]
#[tauri::command(rename = "save_file")]
pub async fn translations_save_file(
  file: &str,
  edits: HashMap<String, Vec<TranslationEdit>>,
  state: State<'_, TranslationProjectState>,
) -> TauriResult<TranslationProjectDescriptor> {
  let (roots, prefix, mode, sources) = {
    let lock = state.project.lock().unwrap();
    let descriptor: &TranslationProjectDescriptor = lock
      .as_ref()
      .ok_or_else(|| String::from("No translations project is open"))?;
    let entry: &TranslationFile = descriptor
      .files
      .get(file)
      .ok_or_else(|| format!("Translations file '{file}' is not part of the open project"))?;

    (
      descriptor.roots.clone(),
      descriptor.prefix.clone(),
      descriptor.mode,
      entry.sources.clone(),
    )
  };

  // Mounted once and kept for the re-read below. Edits replace files in place, so nothing the mount
  // indexed moves, and the default cache policy retains nothing for a stale read to come back from.
  let vfs: XrayVfs = roots.open().map_err(error_to_string)?;
  let scope: XrayLookupScope = XrayLookupScope::all()
    .with_optional_prefix(Some(&prefix))
    .map_err(error_to_string)?;
  let scoped: XrayScopedVfs = vfs.scoped(&scope);

  for (language, language_edits) in &edits {
    if language_edits.is_empty() {
      continue;
    }

    let source: &TranslationSource = sources
      .get(language)
      .ok_or_else(|| format!("Translations file '{file}' has nothing on disk for '{language}'"))?;

    let asset: XrayAsset = scoped
      .find(&source.logical_path)
      .map_err(error_to_string)?
      .ok_or_else(|| {
        format!(
          "Translations file '{}' is no longer in the mounted roots",
          source.logical_path
        )
      })?;

    log::info!(
      "Saving {} edits to {} ({})",
      language_edits.len(),
      source.logical_path,
      language
    );

    apply_edits_to_asset(&asset, language, language_edits).map_err(error_to_string)?;
  }

  // Re-read rather than patch the cached copy: what is on disk now is the only version worth showing,
  // and a write can add or drop entries the caller did not predict.
  let refreshed: TranslationProjectDescriptor = match mode {
    TranslationProjectMode::Source => read_source_in(&vfs, &roots, &prefix),
    TranslationProjectMode::Gamedata => read_gamedata_in(&vfs, &roots, &prefix),
  }
  .map_err(error_to_string)?;

  *state.project.lock().unwrap() = Some(refreshed.clone());

  Ok(refreshed)
}
