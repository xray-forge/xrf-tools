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

  /// Whether the engine reads this kind to decide whether a dialog or phrase is offered.
  pub fn is_condition(&self) -> bool {
    matches!(self, Self::Precondition | Self::HasInfo | Self::DontHasInfo)
  }

  /// Whether this kind names an info portion.
  pub fn is_info_portion(&self) -> bool {
    matches!(
      self,
      Self::GiveInfo | Self::DisableInfo | Self::HasInfo | Self::DontHasInfo
    )
  }

  /// Whether this kind names a script function.
  pub fn is_script_call(&self) -> bool {
    matches!(
      self,
      Self::Precondition | Self::Action | Self::InitFunc | Self::ScriptText
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
