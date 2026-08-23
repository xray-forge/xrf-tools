use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use xrf_vfs::XrayWorldSpec;

use crate::project::mode::DialogProjectMode;

/// Something worth reporting about a project that was opened anyway.
///
/// The reader refuses nothing on content, so this is how an off-schema element or an unreadable file
/// reaches a caller. Phase 4's validation produces `xrf_report::Finding` instead; this is the
/// narrower thing a project open can say about itself.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct DialogFinding {
  pub rule: String,
  pub subject: Option<String>,
  pub message: String,
}

impl DialogFinding {
  pub fn new(rule: impl Into<String>, subject: Option<String>, message: impl Into<String>) -> Self {
    Self {
      rule: rule.into(),
      subject,
      message: message.into(),
    }
  }
}

/// One dialog, as the project index lists it.
///
/// Enough to draw a tree and pick something to open, and deliberately not the phrases: 502 dialogs
/// of those is a payload nobody reads, so a dialog is fetched when it is selected.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct DialogDescriptor {
  pub id: String,
  pub phrases: usize,
  pub priority: Option<i32>,
}

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
  pub dialogs: Vec<DialogDescriptor>,
}

/// An opened dialog project.
///
/// Both prefixes are echoed back rather than left for the caller to re-derive: the mode and any
/// overrides decided them, and a follow-up read that guessed differently would address another tree.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct DialogProjectDescriptor {
  pub mode: DialogProjectMode,
  /// The world this project was opened over, echoed back so a follow-up read addresses the same trees.
  pub world: XrayWorldSpec,
  /// Logical prefix the dialogs were read from.
  pub dialogs_prefix: String,
  /// Logical prefix dialog text is read from.
  pub translations_prefix: String,
  /// Whether every file the project holds is loose, so an editing session could save all of it.
  pub is_editable: bool,
  /// Files keyed by their logical path, in logical-path order.
  pub files: IndexMap<String, DialogFileDescriptor>,
  pub findings: Vec<DialogFinding>,
}
