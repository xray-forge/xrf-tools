use std::cmp::Ordering;

use serde::Serialize;

use crate::RuleId;

/// A single rule violation or checker error.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
  message: String,
  rule_id: RuleId,
  subject: Option<String>,
}

impl Finding {
  pub fn new(rule_id: RuleId, subject: Option<String>, message: impl Into<String>) -> Self {
    Self {
      message: message.into(),
      rule_id,
      subject,
    }
  }

  pub fn message(&self) -> &str {
    &self.message
  }

  pub fn rule_id(&self) -> &RuleId {
    &self.rule_id
  }

  pub fn subject(&self) -> Option<&str> {
    self.subject.as_deref()
  }

  pub(crate) fn cmp(left: &Self, right: &Self) -> Ordering {
    left
      .subject
      .cmp(&right.subject)
      .then_with(|| left.rule_id.cmp(&right.rule_id))
      .then_with(|| left.message.cmp(&right.message))
  }
}
