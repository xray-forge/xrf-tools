use tauri::State;
use xrf_translation::{TranslationProjectDescriptor, TranslationProjectMode, read_gamedata, read_source};
use xrf_vfs::XrayRoots;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;
use crate::plugins::translations::state::TranslationProjectState;

/// Open a translations tree.
///
/// `roots` is the shared vocabulary every surface names roots with, so an installation opens as
/// readily as a loose tree and a gamedata tree layers in front of one. The prefix is this layout's
/// own half — where inside those trees the string tables sit — and defaults to what the mode implies.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_project"))]
#[tauri::command(rename = "open_project")]
pub async fn translations_open_project(
  roots: XrayRoots,
  mode: TranslationProjectMode,
  prefix: Option<String>,
  state: State<'_, TranslationProjectState>,
) -> TauriResult<TranslationProjectDescriptor> {
  let prefix: String = prefix.unwrap_or_else(|| mode.get_prefix().to_owned());

  log::info!(
    "Opening translations project: {} root(s), {:?}, '{}'",
    roots.roots.len(),
    mode,
    prefix
  );

  // The caller's mode is obeyed, not re-derived: the two layouts save to different files, so a guess
  // acted on here would decide what a later save overwrites.
  let descriptor: TranslationProjectDescriptor = match mode {
    TranslationProjectMode::Source => read_source(&roots, &prefix),
    TranslationProjectMode::Gamedata => read_gamedata(&roots, &prefix),
  }
  .map_err(error_to_string)?;

  log::info!(
    "Opened {} translation files, {} languages, {} findings, editable: {}",
    descriptor.files.len(),
    descriptor.languages.len(),
    descriptor.findings.len(),
    descriptor.is_editable
  );

  // Committed only once the read succeeded, so a failed open leaves whatever was already open in place.
  state.open_project(descriptor.clone())?;

  Ok(descriptor)
}
