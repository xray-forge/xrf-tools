use std::sync::Arc;

use xrf_dialog::{DialogListedFiles, DialogPartners, DialogProjectMode};
use xrf_error::XrfResult;
use xrf_ltx::LtxResolution;

use crate::project::gamedata_project::GamedataProject;

/// The dialog partners every check reading characters, profiles or info portions shares.
impl GamedataProject {
  /// The characters, profiles and info portions the engine loads: from the files `system.ltx` lists, or from every
  /// file named as a partner file where it lists none or does not resolve.
  ///
  /// Read on first use and held for the project's life, so the checks asking for them meet one read however many ask
  /// and however they overlap; a check asking while another reads waits for that read.
  ///
  /// # Errors
  ///
  /// Returns an error when the dialogs prefix is not a valid logical path.
  pub(crate) fn get_dialog_partners(&self) -> XrfResult<&DialogPartners> {
    self
      .dialog_partners
      .get_or_init(|| self.read_dialog_partners())
      .as_ref()
      .map_err(Clone::clone)
  }

  /// Reads the partners as [`Self::get_dialog_partners`] describes them, as a dialog project opened over the same lists
  /// reads them (`DialogPartners::from_project`).
  fn read_dialog_partners(&self) -> XrfResult<DialogPartners> {
    let system: Option<Arc<LtxResolution>> = self.ltx_project.system_ltx().ok();
    let listed: Option<DialogListedFiles> = system
      .as_ref()
      .and_then(|system| DialogListedFiles::from_system_ltx(&system.ltx));

    match listed {
      Some(listed) => Ok(DialogPartners::from_listed(
        &self.search,
        DialogProjectMode::DIALOGS_PREFIX,
        &listed,
      )),
      None => DialogPartners::from_search(&self.search, DialogProjectMode::DIALOGS_PREFIX),
    }
  }
}
