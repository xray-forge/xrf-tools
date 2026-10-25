use std::time::Duration;

use serde::Serialize;

use crate::{CheckId, Finding, Status};

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
  pub fn new(id: CheckId, status: Status, duration: Option<Duration>, findings: Vec<Finding>) -> Self {
    let mut findings: Vec<Finding> = findings;

    findings.sort_by(Finding::cmp);

    Self {
      duration,
      findings,
      id,
      status,
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
