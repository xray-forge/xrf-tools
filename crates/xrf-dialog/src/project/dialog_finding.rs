use serde::{Deserialize, Serialize};

/// Something worth reporting about a project that was opened anyway.
///
/// The reader refuses nothing on content, so this is how an off-schema element or an unreadable file
/// reaches a caller: the narrow thing opening a project can say about itself, not a validation of it.
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
