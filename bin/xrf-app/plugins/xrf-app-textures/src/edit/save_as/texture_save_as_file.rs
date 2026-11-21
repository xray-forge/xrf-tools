use serde::Serialize;

use crate::edit::TextureSaveAsRole;

/// One file a copy writes, and the original's file it is made from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TextureSaveAsFile {
  pub role: TextureSaveAsRole,
  /// The original's file, by logical path; `None` for a descriptor the original lacks, which the copy gets new.
  pub from: Option<String>,
  /// The copy's file, by logical path.
  pub to: String,
  /// Whether a file with this name already exists; it is replaced only when the save is forced.
  pub is_replacing: bool,
}
