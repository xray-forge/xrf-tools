use std::time::Duration;

use serde::Serialize;

use crate::check_id::CheckId;
use crate::finding::Finding;
use crate::status::Status;

/// Immutable report data for one command check.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckReport {
  /// `null` when the check did not measure itself; a millisecond count otherwise, like every other
  /// duration on the wire. Never omitted, because the IPC bindings cannot type a conditionally skipped
  /// field.
  #[serde(with = "xrf_utils::optional_duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = Option<u64>))]
  duration: Option<Duration>,
  findings: Vec<Finding>,
  id: CheckId,
  status: Status,
}

impl CheckReport {
  /// A check's report, its findings in their report order ([`Finding`]'s `Ord`) whatever order they were found in.
  ///
  /// A passed check holds warnings at most: a failure is what fails a check, so a producer reporting one beside a pass
  /// has its verdict and its findings disagreeing, which a reader trusting either would be misled by.
  pub fn new(id: CheckId, status: Status, duration: Option<Duration>, findings: Vec<Finding>) -> Self {
    debug_assert!(
      status != Status::Passed || !findings.iter().any(Finding::is_failure),
      "check '{id}' passed while holding a failure finding"
    );

    let mut findings: Vec<Finding> = findings;

    findings.sort();

    Self {
      duration,
      findings,
      id,
      status,
    }
  }

  /// A check that judges by its findings alone, its verdict [`Status::from_findings`].
  pub fn judge(id: CheckId, duration: Option<Duration>, findings: Vec<Finding>) -> Self {
    let status: Status = Status::from_findings(&findings);

    Self::new(id, status, duration, findings)
  }

  /// The check as a strict run judges it: one that passed with warnings fails, every other verdict stands.
  pub fn into_strict(self) -> Self {
    if self.status == Status::Passed && !self.findings.is_empty() {
      Self {
        status: Status::Failed,
        ..self
      }
    } else {
      self
    }
  }

  pub const fn duration(&self) -> Option<Duration> {
    self.duration
  }

  pub fn findings(&self) -> &[Finding] {
    &self.findings
  }

  pub fn id(&self) -> &CheckId {
    &self.id
  }

  pub const fn status(&self) -> Status {
    self.status
  }
}
