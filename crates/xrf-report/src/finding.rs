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
}

/// The order a check reports its findings in: by subject, with the check's own findings (no subject) first, then by
/// rule, then by message.
impl Ord for Finding {
  fn cmp(&self, other: &Self) -> Ordering {
    self
      .subject
      .cmp(&other.subject)
      .then_with(|| self.rule_id.cmp(&other.rule_id))
      .then_with(|| self.message.cmp(&other.message))
  }
}

impl PartialOrd for Finding {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}
