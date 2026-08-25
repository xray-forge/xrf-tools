use std::ops::Range;

use serde::{Deserialize, Serialize};

/// What a dialog or phrase child element means to the engine.
///
/// Classification only. The element keeps the name it was written with, so an element this does not
/// recognise still survives a round trip; mods add their own, and one shipped project uses a
/// `go_back` phrase element the engine never defined.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum DialogElementKind {
  /// Translation key of the line, not the line itself.
  Text,
  /// Script call producing the line at runtime, in place of a translation key.
  ScriptText,
  /// Script call run when the phrase is selected.
  Action,
  /// Script predicate gating visibility.
  Precondition,
  /// Id of a phrase that may follow this one.
  Next,
  /// Info portion granted.
  GiveInfo,
  /// Info portion revoked.
  DisableInfo,
  /// Info portion required.
  HasInfo,
  /// Info portion that must be absent.
  DontHasInfo,
  /// Whether selecting the phrase ends the conversation.
  IsFinal,
  /// Script call run when the dialog is initialised.
  InitFunc,
  /// Recognised container rather than a value: `phrase_list` or `phrase`.
  Container,
  /// Not part of the schema. Preserved and reported.
  Unknown,
}

impl DialogElementKind {
  /// Classify an element name, answering `Unknown` for a name the schema does not define.
  pub fn from_name(name: &str) -> Self {
    match name {
      "text" => Self::Text,
      "script_text" => Self::ScriptText,
      "action" => Self::Action,
      "precondition" => Self::Precondition,
      "next" => Self::Next,
      "give_info" => Self::GiveInfo,
      "disable_info" => Self::DisableInfo,
      "has_info" => Self::HasInfo,
      "dont_has_info" => Self::DontHasInfo,
      "is_final" => Self::IsFinal,
      "init_func" => Self::InitFunc,
      "phrase_list" | "phrase" => Self::Container,
      _ => Self::Unknown,
    }
  }

  /// Whether a `dialog` may hold this kind.
  pub fn is_valid_for_dialog(&self) -> bool {
    matches!(
      self,
      Self::Precondition | Self::InitFunc | Self::HasInfo | Self::DontHasInfo | Self::Action | Self::Container
    )
  }

  /// Whether a `phrase` may hold this kind.
  pub fn is_valid_for_phrase(&self) -> bool {
    matches!(
      self,
      Self::Text
        | Self::ScriptText
        | Self::Action
        | Self::Precondition
        | Self::Next
        | Self::GiveInfo
        | Self::DisableInfo
        | Self::HasInfo
        | Self::DontHasInfo
        | Self::IsFinal
    )
  }
}

/// One child element of a dialog or a phrase, as written.
///
/// Elements are held in an ordered list rather than as named fields because nearly every one of them
/// repeats, and because `next` order decides the order the player sees the options in. A struct of
/// `Option` fields would lose both facts, and a set would lose the second silently.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DialogElement {
  name: String,
  value: String,
  range: Range<usize>,
}

impl DialogElement {
  pub fn new(name: String, value: String, range: Range<usize>) -> Self {
    Self { name, value, range }
  }

  /// The element name as written, which is what a rewrite has to reproduce.
  pub fn get_name(&self) -> &str {
    &self.name
  }

  /// Text content with entity references resolved.
  pub fn get_value(&self) -> &str {
    &self.value
  }

  /// Byte range of the whole element in the document source.
  pub fn get_range(&self) -> &Range<usize> {
    &self.range
  }

  pub fn get_kind(&self) -> DialogElementKind {
    DialogElementKind::from_name(&self.name)
  }

  /// Whether the value reads as set.
  ///
  /// The engine treats any content as true for a flag element; shipped data writes `1`.
  pub fn is_enabled(&self) -> bool {
    !self.value.trim().is_empty() && self.value.trim() != "0"
  }
}
