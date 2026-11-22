use std::collections::HashMap;
use std::path::PathBuf;

use xrf_dialog::{Dialog, DialogElement, DialogElementKind, DialogProject};

use crate::project::dialogs::dialog_phrase_graph::DialogPhraseGraph;
use crate::project::dialogs::dialogs_verifier::DialogsVerifier;
use crate::project::dialogs::verify_dialogs_result::GamedataDialogsVerificationResult;
use crate::project::gamedata_verification_rule::GamedataVerificationRule;
use crate::project::scripts::gamedata_script_functions::GamedataScriptFunctions;

/// A script call a dialog makes, with the dialog or phrase it is written on.
struct CalledFunction<'a> {
  owner: String,
  element: &'a DialogElement,
}

/// The script functions dialogs name: preconditions, actions, script texts and the `init_func` of a scripted dialog.
impl DialogsVerifier<'_> {
  /// Every script function a dialog the engine reads names, itself or in a phrase it reaches, is defined.
  ///
  /// `CDialogScriptHelper` throws on a precondition, action or script text the script engine cannot find when it
  /// evaluates it, and `CPhraseDialog::load_shared` on a missing `init_func` when it builds the dialog; a release build
  /// skips the throw and calls nothing, which fails as surely. A condition of phrase `0` the engine never reads, and an
  /// `init_func` beside a phrase list, are not judged, and nothing is in a tree holding no scripts.
  pub(super) fn verify_functors(
    &self,
    project: &DialogProject,
    declared: &HashMap<&str, &Dialog>,
    result: &mut GamedataDialogsVerificationResult,
  ) {
    let Some(functions) = GamedataScriptFunctions::read(self.project, &self.options.job) else {
      return;
    };

    for file in project.get_files() {
      let path: PathBuf = self.report_path(file.get_logical_path());

      for dialog in file.get_file().get_dialogs() {
        if !Self::is_declaration_read(declared, dialog) {
          continue;
        }

        for called in Self::list_called(dialog) {
          let name: &str = called.element.get_value().trim();

          result.checked_references_count += 1;

          if functions.is_missing(name) {
            result.missing_references_count += 1;
            result.fail(
              GamedataVerificationRule::DialogFunctorMissing,
              &path,
              format!(
                "{} {} names script function '{name}', which no script defines",
                called.owner,
                called.element.get_name()
              ),
            );
          }
        }
      }
    }
  }

  /// The script calls the engine reads from a dialog, each with the dialog or phrase it is written on.
  fn list_called(dialog: &Dialog) -> Vec<CalledFunction<'_>> {
    let id: &str = dialog.get_id();
    let graph: DialogPhraseGraph<'_> = DialogPhraseGraph::new(dialog);
    let is_scripted: bool = graph.is_scripted();
    let mut called: Vec<CalledFunction<'_>> = dialog
      .get_elements()
      .iter()
      .filter(|element| element.get_kind().is_script_call())
      .filter(|element| is_scripted || element.get_kind() != DialogElementKind::InitFunc)
      .map(|element| CalledFunction {
        owner: format!("Dialog '{id}'"),
        element,
      })
      .collect();

    let is_entry_read: bool = dialog.is_entry_phrase_revisited();

    for phrase in graph.list_reached() {
      let is_entry: bool = phrase.get_id() == Dialog::ENTRY_PHRASE_ID;

      called.extend(
        phrase
          .get_elements()
          .iter()
          .filter(|element| element.get_kind().is_script_call())
          .filter(|element| is_entry_read || !is_entry || !element.get_kind().is_condition())
          .map(|element| CalledFunction {
            owner: format!("Dialog '{id}' phrase '{}'", phrase.get_id()),
            element,
          }),
      );
    }

    called
  }
}
