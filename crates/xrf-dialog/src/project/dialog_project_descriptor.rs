use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use xrf_vfs::XrayRoots;

use crate::project::dialog_file_descriptor::DialogFileDescriptor;
use crate::project::dialog_finding::DialogFinding;
use crate::project::mode::DialogProjectMode;

/// An opened dialog project.
///
/// Both prefixes are echoed back rather than left for the caller to re-derive: the mode and any
/// overrides decided them, and a follow-up read that guessed differently would address another tree.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct DialogProjectDescriptor {
  pub mode: DialogProjectMode,
  /// The roots this project was opened over, echoed back so a follow-up read addresses the same trees.
  pub roots: XrayRoots,
  /// Logical prefix the dialogs were read from.
  pub dialogs_prefix: String,
  /// Logical prefix dialog text is read from.
  pub translations_prefix: String,
  /// Whether every file the project holds is loose, so an editing session could save all of it.
  pub is_editable: bool,
  /// Languages the text tree offers, which is what a language switcher is built from.
  pub languages: Vec<String>,
  /// Distinct translation keys the text tree defines.
  pub text_keys: usize,
  /// Files keyed by their logical path, in logical-path order.
  pub files: IndexMap<String, DialogFileDescriptor>,
  pub findings: Vec<DialogFinding>,
}
