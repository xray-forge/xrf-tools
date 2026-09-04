use crate::dltx_severity::DltxSeverity;

/// Something worth saying about a patched config tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DltxDiagnostic {
  pub severity: DltxSeverity,
  /// Section the finding is about, lowercased as the engine stores it.
  pub section: String,
  /// File that raised it, when one statement is responsible.
  pub file: Option<String>,
  pub message: String,
  /// What the engine itself does here, present when that differs from reporting it.
  ///
  /// Recorded because a warning about behaviour nobody can observe in game is read differently from one about a
  /// crash: a modder needs to know the game will load this and stay quiet.
  pub engine_behaviour: Option<String>,
}

impl DltxDiagnostic {
  pub fn new_error(section: impl Into<String>, message: impl Into<String>) -> Self {
    Self {
      engine_behaviour: None,
      file: None,
      message: message.into(),
      section: section.into(),
      severity: DltxSeverity::Error,
    }
  }

  pub fn new_warning(section: impl Into<String>, message: impl Into<String>) -> Self {
    Self {
      engine_behaviour: None,
      file: None,
      message: message.into(),
      section: section.into(),
      severity: DltxSeverity::Warning,
    }
  }

  /// Records what the engine does with the same input.
  pub fn with_engine_behaviour(mut self, behaviour: impl Into<String>) -> Self {
    self.engine_behaviour = Some(behaviour.into());

    self
  }

  pub fn with_file(mut self, file: impl Into<String>) -> Self {
    self.file = Some(file.into());

    self
  }
}
