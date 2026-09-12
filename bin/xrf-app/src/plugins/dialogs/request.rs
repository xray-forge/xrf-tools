use serde::{Deserialize, Serialize};
use xrf_dialog::DialogProjectMode;
use xrf_vfs::XrayRoots;

use crate::core::session::SessionId;

/// What opening a dialogs project was asked to do.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct DialogsOpenRequest {
  pub session_id: SessionId,
  /// Trees to search, and how each is read.
  pub roots: XrayRoots,
  /// How much of the project to read.
  pub mode: DialogProjectMode,
  /// Scope holding the dialog files, or nothing for all of them.
  pub dialogs_prefix: Option<String>,
  /// Scope holding the string tables, or nothing for all of them.
  pub translations_prefix: Option<String>,
}

/// Identifies a dialog within one committed project.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct DialogsReadRequest {
  pub session_id: SessionId,
  pub logical_path: String,
  pub id: String,
  pub language: Option<String>,
}
