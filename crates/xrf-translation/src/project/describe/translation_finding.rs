use serde::{Deserialize, Serialize};

/// Something worth reporting about a file that was opened anyway.
///
/// The reader refuses nothing on content: an editor that will not open the file you need to fix is
/// no use, and the build and verifier keep their own guards.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationFinding {
  pub rule: String,
  pub subject: Option<String>,
  pub message: String,
}

impl TranslationFinding {
  pub fn new(rule: impl Into<String>, subject: Option<String>, message: impl Into<String>) -> Self {
    Self {
      rule: rule.into(),
      subject,
      message: message.into(),
    }
  }
}
