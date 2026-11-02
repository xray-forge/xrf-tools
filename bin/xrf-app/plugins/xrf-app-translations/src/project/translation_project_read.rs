use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionDerived, SessionSnapshot};
use xrf_translation::{
  TranslationGamedataReader, TranslationProjectDescriptor, TranslationProjectMode, TranslationSourceReader,
};
use xrf_utils::error_to_string;
use xrf_vfs::{XrayInputRecord, XrayRoots, XraySearch};

/// How a translations project is read through the shared roots, recording every file and listing it reads: what an
/// open commits, and what a commit's follow derives again.
pub struct TranslationProjectRead;

impl TranslationProjectRead {
  /// Reads a translations project through `search`. The caller's mode is obeyed, not re-derived: the two layouts save
  /// to different files.
  ///
  /// # Errors
  ///
  /// Returns an error when the prefix is not a logical path, or the project cannot be read.
  pub fn read(
    search: XraySearch,
    roots: &XrayRoots,
    mode: TranslationProjectMode,
    prefix: &str,
  ) -> TauriResult<SessionDerived<TranslationProjectDescriptor>> {
    let inputs: XrayInputRecord = XrayInputRecord::new();
    let search: XraySearch = search.with_recording(&inputs);
    let descriptor: TranslationProjectDescriptor = match mode {
      TranslationProjectMode::Source => TranslationSourceReader::read_in(&search, roots, prefix),
      TranslationProjectMode::Gamedata => TranslationGamedataReader::read_in(&search, roots, prefix),
    }
    .map_err(error_to_string)?;

    Ok(SessionDerived::new(descriptor, inputs))
  }

  /// The open project read again over the roots it was opened over, once a commit moved a file it read.
  ///
  /// # Errors
  ///
  /// As [`Self::read`], or when the roots cannot be searched.
  pub fn derive(
    project: &SessionSnapshot<TranslationProjectDescriptor>,
    assets: &AssetMountState,
  ) -> TauriResult<SessionDerived<TranslationProjectDescriptor>> {
    Self::read(
      assets.search(&project.roots)?,
      &project.roots,
      project.mode,
      &project.prefix,
    )
  }
}
