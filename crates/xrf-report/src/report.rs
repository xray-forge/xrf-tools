use serde::Serialize;

use crate::{CheckReport, Status};

/// Immutable, finalized account of a command's checks and findings.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
  checks: Vec<CheckReport>,
  status: Status,
}

impl Report {
  /// Creates a report while preserving the caller's check order.
  pub fn new(checks: Vec<CheckReport>) -> Self {
    let status: Status = Status::aggregate(checks.iter().map(|check| check.status()));

    Self { checks, status }
  }

  pub fn checks(&self) -> &[CheckReport] {
    &self.checks
  }

  /// How many findings every check made together.
  pub fn count_findings(&self) -> usize {
    self.checks.iter().map(|check| check.findings().len()).sum()
  }

  pub const fn status(&self) -> Status {
    self.status
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;

  use super::{CheckReport, Report, Status};
  use crate::{CheckId, Finding, RuleId};

  #[test]
  fn preserves_check_order_and_uses_pre_finalized_checks() {
    let first_check: CheckReport = CheckReport::new(
      CheckId::new("textures").unwrap(),
      Status::Failed,
      Some(Duration::ZERO),
      vec![
        Finding::new(
          RuleId::new("textures.read").unwrap(),
          Some(String::from("z.dds")),
          "same",
        ),
        Finding::new(
          RuleId::new("textures.format").unwrap(),
          Some(String::from("a.dds")),
          "same",
        ),
        Finding::new(
          RuleId::new("textures.read").unwrap(),
          Some(String::from("a.dds")),
          "same",
        ),
      ],
    );
    let second_check: CheckReport = CheckReport::new(
      CheckId::new("scripts").unwrap(),
      Status::Passed,
      Some(Duration::ZERO),
      Vec::new(),
    );

    assert_eq!(first_check.findings()[0].rule_id().as_str(), "textures.format");
    assert_eq!(first_check.findings()[1].rule_id().as_str(), "textures.read");
    assert_eq!(first_check.findings()[2].subject(), Some("z.dds"));

    let report: Report = Report::new(vec![first_check, second_check]);

    assert_eq!(report.checks()[0].id().as_str(), "textures");
    assert_eq!(report.checks()[1].id().as_str(), "scripts");
    assert_eq!(report.count_findings(), 3);
  }
}
