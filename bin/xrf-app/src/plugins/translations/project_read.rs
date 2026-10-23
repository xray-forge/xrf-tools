use xrf_translation::{TranslationProjectDescriptor, TranslationProjectMode, read_gamedata_in, read_source_in};
use xrf_utils::error_to_string;
use xrf_vfs::{XrayInputRecord, XrayRoots, XraySearch};

use crate::core::assets::AssetMountState;
use crate::core::session::{SessionDerived, SessionSnapshot};
use crate::core::types::TauriResult;

/// Reads a translations project through `search`, recording every file and listing it reads: what an open commits, and
/// what a commit's follow derives again.
pub(in crate::plugins::translations) fn read_project(
  search: XraySearch,
  roots: &XrayRoots,
  mode: TranslationProjectMode,
  prefix: &str,
) -> TauriResult<SessionDerived<TranslationProjectDescriptor>> {
  let inputs: XrayInputRecord = XrayInputRecord::new();
  let search: XraySearch = search.with_recording(&inputs);
  let descriptor: TranslationProjectDescriptor = match mode {
    TranslationProjectMode::Source => read_source_in(&search, roots, prefix),
    TranslationProjectMode::Gamedata => read_gamedata_in(&search, roots, prefix),
  }
  .map_err(error_to_string)?;

  Ok(SessionDerived::new(descriptor, inputs))
}

/// The open project read again over the roots it was opened over, once a commit moved a file it read.
pub(in crate::plugins::translations) fn derive_project(
  project: &SessionSnapshot<TranslationProjectDescriptor>,
  assets: &AssetMountState,
) -> TauriResult<SessionDerived<TranslationProjectDescriptor>> {
  read_project(
    assets.search(&project.roots)?,
    &project.roots,
    project.mode,
    &project.prefix,
  )
}
