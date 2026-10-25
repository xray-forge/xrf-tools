use serde::{Deserialize, Serialize};

use crate::project::dialog_summary_descriptor::DialogSummaryDescriptor;

/// One dialog file the project holds.
///
/// Keyed by its logical path, so the key is the engine identity and the value says where that identity
/// was actually found.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct DialogFileDescriptor {
  /// Host path when the winner is a loose file; absent when it comes out of an archive.
  pub physical_path: Option<String>,
  /// Whether an edit could write this file back. False for an archived winner.
  pub is_editable: bool,
  /// The code page the file was decoded with, and the one a rewrite has to use.
  pub encoding: String,
  pub dialogs: Vec<DialogSummaryDescriptor>,
}
