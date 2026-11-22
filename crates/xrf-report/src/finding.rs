use std::cmp::Ordering;

use serde::Serialize;

use crate::rule_id::RuleId;
use crate::severity::Severity;

/// A single rule violation or checker error, and whether it fails its check.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
  message: String,
  rule_id: RuleId,
  severity: Severity,
  subject: Option<String>,
}

impl Finding {
  /// A finding that fails its check.
  pub fn failure(rule_id: RuleId, subject: Option<String>, message: impl Into<String>) -> Self {
    Self {
      message: message.into(),
      rule_id,
      severity: Severity::Failure,
      subject,
    }
  }

  /// A finding that is reported, and fails its check only when the run is strict.
  pub fn warning(rule_id: RuleId, subject: Option<String>, message: impl Into<String>) -> Self {
    Self {
      severity: Severity::Warning,
      ..Self::failure(rule_id, subject, message)
    }
  }

  /// The same finding about `subject` instead, for a reader that names subjects its own way (relative to a root).
  pub fn with_subject(self, subject: Option<String>) -> Self {
    Self { subject, ..self }
  }

  pub fn message(&self) -> &str {
    &self.message
  }

  pub fn rule_id(&self) -> &RuleId {
    &self.rule_id
  }

  pub const fn severity(&self) -> Severity {
    self.severity
  }

  pub fn subject(&self) -> Option<&str> {
    self.subject.as_deref()
  }

  pub const fn is_failure(&self) -> bool {
    matches!(self.severity, Severity::Failure)
  }
}

/// The order a check reports its findings in: by subject, with the check's own findings (no subject) first, then by
/// rule, then by message, then failures before warnings.
impl Ord for Finding {
  fn cmp(&self, other: &Self) -> Ordering {
    self
      .subject
      .cmp(&other.subject)
      .then_with(|| self.rule_id.cmp(&other.rule_id))
      .then_with(|| self.message.cmp(&other.message))
      .then_with(|| self.severity.cmp(&other.severity))
  }
}

impl PartialOrd for Finding {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}
