use crate::project::dialog_project_mode::DialogProjectMode;

/// Where inside roots this domain's data sits.
///
/// The other half of opening a project. A roots says which trees are searched and in what order;
/// this says which logical prefixes inside them hold dialogs and dialog text. Every tool shares the
/// first question and answers the second for itself.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DialogProjectLayout {
  pub mode: DialogProjectMode,
  /// Stands in for the prefix the mode would default to.
  pub dialogs_prefix: Option<String>,
  pub translations_prefix: Option<String>,
  /// Whether the text tree goes unread, for a caller that shows no lines: phrases then show their keys.
  pub is_text_skipped: bool,
}

impl DialogProjectLayout {
  pub fn new(mode: DialogProjectMode) -> Self {
    Self {
      mode,
      dialogs_prefix: None,
      translations_prefix: None,
      is_text_skipped: false,
    }
  }

  /// The prefix dialogs are read from.
  pub fn get_dialogs_prefix(&self) -> &str {
    self
      .dialogs_prefix
      .as_deref()
      .unwrap_or(DialogProjectMode::DIALOGS_PREFIX)
  }

  /// The prefix dialog text is read from.
  pub fn get_translations_prefix(&self) -> &str {
    self
      .translations_prefix
      .as_deref()
      .unwrap_or_else(|| self.mode.get_translations_prefix())
  }
}
