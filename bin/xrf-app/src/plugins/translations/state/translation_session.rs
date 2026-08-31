use xrf_translation::TranslationProjectDescriptor;

use crate::core::types::TauriResult;
use crate::plugins::translations::state::translation_session_id::TranslationSessionId;

/// What is open, paired with the identity of the open that made it so.
pub(super) struct TranslationSession {
  pub id: TranslationSessionId,
  pub project: Option<TranslationProjectDescriptor>,
}

impl TranslationSession {
  pub(super) fn new() -> Self {
    Self {
      id: TranslationSessionId::new(),
      project: None,
    }
  }

  /// Start a new session over `project`, which is `None` for a close.
  pub(super) fn restart(&mut self, project: Option<TranslationProjectDescriptor>) {
    self.id = TranslationSessionId::new();
    self.project = project;
  }

  pub(super) fn require_project(&self) -> TauriResult<&TranslationProjectDescriptor> {
    self
      .project
      .as_ref()
      .ok_or_else(|| String::from("No translations project is open"))
  }
}
