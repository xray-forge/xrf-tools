use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

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
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct DialogFileDescriptor {
  /// Absolute path on disk, which is what a later write has to reach.
  pub path: String,
  /// The code page the file was decoded with, and the one a rewrite has to use.
  pub encoding: String,
  pub dialogs: Vec<DialogDescriptor>,
}

/// An opened dialog project.
///
/// Both roots are echoed back rather than left for the caller to re-derive: the mode and any
/// overrides decided them, and a follow-up read that guessed differently would address another tree.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct DialogProjectDescriptor {
  pub mode: DialogProjectMode,
  pub root: String,
  pub dialogs_root: String,
  pub translations_root: String,
  /// Files keyed by their path relative to the dialogs root, in discovery order.
  pub files: IndexMap<String, DialogFileDescriptor>,
  pub findings: Vec<DialogFinding>,
}
