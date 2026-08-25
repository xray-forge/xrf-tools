use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use xrf_vfs::XrayRoots;

use crate::dialog::Dialog;
use crate::element::{DialogElement, DialogElementKind};
use crate::phrase::DialogPhrase;
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
///
/// Named for the reduction it is. A descriptor carrying a domain type's own name mirrors that type —
/// [`DialogDescriptor`] mirrors `Dialog` — so a summary of one has to say so.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct DialogSummaryDescriptor {
  pub id: String,
  pub phrases: usize,
  pub priority: Option<i32>,
}

/// One child element of a dialog or a phrase, as written.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct DialogElementDescriptor {
  /// The element name as written, such as `give_info`.
  pub name: String,
  /// What that name means to the engine.
  pub kind: DialogElementKind,
  /// Text content, with entity references already resolved.
  pub value: String,
}

impl From<&DialogElement> for DialogElementDescriptor {
  fn from(element: &DialogElement) -> Self {
    Self {
      name: element.get_name().to_owned(),
      kind: element.get_kind(),
      value: element.get_value().to_owned(),
    }
  }
}

/// One line of a conversation, as a canvas draws it.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct DialogPhraseDescriptor {
  /// Unique within its dialog, and what `next` references. The entry phrase is `0`.
  pub id: String,
  /// Translation key of the line, not the line itself.
  ///
  /// Absent for a phrase whose line comes from `script_text`, which is a state and not a defect:
  /// Anomaly does it 107 times.
  pub text: Option<String>,
  /// Whether selecting this phrase ends the conversation.
  pub is_final: bool,
  /// Whether the phrase sits inside a `phrase_list` rather than directly under its dialog.
  ///
  /// Both forms occur, and a later insertion has to reproduce the one the file already uses.
  pub is_in_phrase_list: bool,
  /// Ids that may follow this one, **in the order the player is offered them**.
  pub next: Vec<String>,
  /// Every child element in document order, including the ones projected above.
  pub elements: Vec<DialogElementDescriptor>,
}

impl From<&DialogPhrase> for DialogPhraseDescriptor {
  fn from(phrase: &DialogPhrase) -> Self {
    Self {
      id: phrase.get_id().to_owned(),
      text: phrase.get_text().map(str::to_owned),
      is_final: phrase.is_final(),
      is_in_phrase_list: phrase.is_in_phrase_list(),
      next: phrase.list_next().into_iter().map(str::to_owned).collect(),
      elements: phrase.get_elements().iter().map(Into::into).collect(),
    }
  }
}

/// One whole dialog: its own elements and every phrase it declares.
///
/// What a selection fetches, against the summary the project index already gave. Both names it was
/// addressed by are echoed back, so a response arriving late cannot be read as another dialog's — the
/// same rule the asset commands follow.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct DialogDescriptor {
  /// Logical path of the file holding it, as the project keys that file.
  pub logical_path: String,
  pub id: String,
  /// Selection priority, negative for a dialog meant to sort last.
  pub priority: Option<i32>,
  /// Dialog-level elements — preconditions, info gates, `init_func` — excluding the phrases.
  pub elements: Vec<DialogElementDescriptor>,
  /// Phrases in document order.
  ///
  /// Empty is legitimate: `dm_traveler_dialog` carries only a precondition and an init function and
  /// builds its phrases from script at runtime.
  pub phrases: Vec<DialogPhraseDescriptor>,
}

impl DialogDescriptor {
  /// Describe a dialog found under a logical path.
  pub fn new(logical_path: impl Into<String>, dialog: &Dialog) -> Self {
    Self {
      logical_path: logical_path.into(),
      id: dialog.get_id().to_owned(),
      priority: dialog.get_priority(),
      elements: dialog.get_elements().iter().map(Into::into).collect(),
      phrases: dialog.get_phrases().iter().map(Into::into).collect(),
    }
  }
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
  pub dialogs: Vec<DialogSummaryDescriptor>,
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
  /// The roots this project was opened over, echoed back so a follow-up read addresses the same trees.
  pub roots: XrayRoots,
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
