use std::collections::HashMap;
use std::path::PathBuf;

use xrf_dialog::{Dialog, DialogListedFiles, DialogPartners, DialogProject, DialogProjectMode};
use xrf_error::XrfResult;
use xrf_vfs::XraySearch;

use crate::project::dialogs::dialogs_verifier::DialogsVerifier;
use crate::project::dialogs::verify_dialogs_result::GamedataDialogsVerificationResult;
use crate::project::gamedata_verification_rule::GamedataVerificationRule;

/// Which files the engine loads: the dialog and partner files `system.ltx` lists, the ones it leaves out, and ids a
/// second file declares again.
impl DialogsVerifier<'_> {
  /// Dialog files the engine cannot load or never loads.
  ///
  /// A listed file the tree lacks stops `CPhraseDialog`'s id index, which loads its list with `crashOnFail`. A file
  /// that does not read is only reported: the engine asserts on one its parser refuses, but this reader may refuse
  /// more than it does.
  pub(super) fn report_dialog_files(
    &self,
    project: &DialogProject,
    listed: Option<&DialogListedFiles>,
    result: &mut GamedataDialogsVerificationResult,
  ) -> XrfResult {
    let prefix: &str = DialogProjectMode::DIALOGS_PREFIX;

    for finding in project.get_findings() {
      let path: PathBuf = self.report_path(finding.subject.as_deref().unwrap_or_default());

      if finding.rule == DialogProject::UNREADABLE_RULE {
        result.tolerate(
          GamedataVerificationRule::DialogUnreadable,
          &path,
          format!(
            "Dialog file does not read, so its dialogs are not judged: {}",
            finding.message
          ),
        );
      } else if finding.rule == DialogProject::MISSING_RULE {
        result.fail(
          GamedataVerificationRule::DialogFileMissing,
          &path,
          String::from(
            "Dialog file is listed in [dialogs] files, but the tree does not hold it, which the engine asserts on",
          ),
        );
      }
    }

    let Some(listed) = listed else {
      return Ok(());
    };

    let search: XraySearch = self.project.search().clone().with_prefix(prefix)?;

    for asset in DialogProject::list_dialog_assets(&search) {
      let logical_path: &str = asset.get_logical_path().as_str();

      if !listed.is_dialog_file(prefix, logical_path) {
        result.tolerate(
          GamedataVerificationRule::DialogFileUnlisted,
          &self.report_path(logical_path),
          String::from("Dialog file is not listed in [dialogs] files, so the engine never loads its dialogs"),
        );
      }
    }

    Ok(())
  }

  /// Every declaration of a dialog id after the first, which the engine's id index skips with only a log line.
  pub(super) fn report_duplicate_dialogs(
    &self,
    project: &DialogProject,
    declared: &HashMap<&str, &Dialog>,
    result: &mut GamedataDialogsVerificationResult,
  ) {
    for file in project.get_files() {
      for dialog in file.get_file().get_dialogs() {
        if !Self::is_declaration_read(declared, dialog) {
          result.tolerate(
            GamedataVerificationRule::DialogDuplicate,
            &self.report_path(file.get_logical_path()),
            format!(
              "Dialog '{}' is declared again, and the first declaration is the one the engine reads",
              dialog.get_id()
            ),
          );
        }
      }
    }
  }

  /// Partner files the engine cannot load or never loads, and partner ids declared twice.
  ///
  /// A listed character or profile file the tree lacks stops the engine's id index, which loads those lists with
  /// `crashOnFail`; the info portion list is loaded without it, so a missing info portion file is skipped. A file that
  /// does not read is only reported, as a dialog file is.
  pub(super) fn report_partner_files(
    &self,
    partners: &DialogPartners,
    listed: Option<&DialogListedFiles>,
    result: &mut GamedataDialogsVerificationResult,
  ) -> XrfResult {
    let prefix: &str = DialogProjectMode::DIALOGS_PREFIX;

    for finding in partners.get_findings() {
      let subject: &str = finding.subject.as_deref().unwrap_or_default();
      let path: PathBuf = self.report_path(subject);

      if finding.rule == DialogPartners::UNREADABLE_RULE {
        result.tolerate(
          GamedataVerificationRule::DialogPartnerUnreadable,
          &path,
          format!(
            "Partner file does not read, so what it declares is not judged: {}",
            finding.message
          ),
        );
      } else if finding.rule == DialogPartners::DUPLICATE_RULE {
        result.tolerate(
          GamedataVerificationRule::DialogDuplicate,
          &path,
          finding.message.clone(),
        );
      } else if finding.rule == DialogPartners::MISSING_RULE {
        if listed.is_some_and(|listed| listed.is_info_portion_file(prefix, subject)) {
          result.tolerate(
            GamedataVerificationRule::DialogPartnerUnreadable,
            &path,
            String::from(
              "Info portion file is listed in system.ltx, but the tree does not hold it, so the engine skips it",
            ),
          );
        } else {
          result.fail(
            GamedataVerificationRule::DialogFileMissing,
            &path,
            String::from(
              "Partner file is listed in system.ltx, but the tree does not hold it, which the engine asserts on",
            ),
          );
        }
      }
    }

    let Some(listed) = listed else {
      return Ok(());
    };

    for logical_path in DialogPartners::list_named_files(self.project.search(), prefix)? {
      if !listed.is_partner_file(prefix, &logical_path) {
        result.tolerate(
          GamedataVerificationRule::DialogFileUnlisted,
          &self.report_path(&logical_path),
          String::from("Partner file is not listed in system.ltx, so the engine never loads what it declares"),
        );
      }
    }

    Ok(())
  }
}
