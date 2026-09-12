use serde::Serialize;
use xrf_ltx_inspect::{LtxAnchoredFinding, LtxFileStructure, LtxFileText, LtxInventory};
use xrf_vfs::XrayRoots;

use crate::core::session::SessionId;

/// What one open of the configs explorer answers with.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigsProjectDescriptor {
  /// Identity every later read is addressed by.
  pub session_id: SessionId,
  /// The trees this project searched, as the backend resolved them, so a reload restores the same open.
  pub roots: XrayRoots,
  /// Scope inside those trees, or nothing for all of them.
  pub prefix: Option<String>,
  /// Whether configs resolve under the Monolith/Anomaly patch dialect.
  pub is_dltx: bool,
  /// Host path the project reports itself at, for a crumb that names something a person recognises.
  pub root: String,
  /// Every config the project holds, and what each one is to it.
  pub inventory: LtxInventory,
  /// Section schemes the project's `*.scheme.ltx` files declare, by name.
  ///
  /// Sent once with the open rather than per document: a tree declares tens of them and every structure read joins
  /// against the same set.
  pub declared_schemes: Vec<String>,
}

/// One config as the authored view renders it.
///
/// Text and structure travel together but stay separate records: the text is what a person edits and the structure is
/// what only the parser knows, and a future edit replaces one without invalidating the shape of the other.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigsDocument {
  pub text: LtxFileText,
  pub structure: LtxFileStructure,
  /// What is wrong with this file itself: it will not parse, or an `#include` reached nothing.
  pub findings: Vec<LtxAnchoredFinding>,
}
