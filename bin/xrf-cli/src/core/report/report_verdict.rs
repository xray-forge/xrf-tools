use std::collections::BTreeMap;
use std::fmt::Display;

use xrf_error::XrfError;
use xrf_output::OutputOptions;
use xrf_report::{Report, Status};

use crate::core::command::{CommandError, CommandResult};

/// How a verify command tells a person what its checks found and turns their report into its outcome: one ladder and
/// one printer for every such command, so their consoles and exit codes agree.
pub struct ReportVerdict;

impl ReportVerdict {
  /// Prints each check that found something, and its findings at verbose level: a sweep of thousands of files can find
  /// more than a terminal is useful for, and the report is the artifact meant for reading them.
  ///
  /// The channel follows the check's verdict: a failed check on the failure channel, one that broke on the error
  /// channel, and one whose findings do not fail it (passed with warnings, incomplete, skipped) as a warning.
  pub fn print_findings(output: &OutputOptions, report: &Report) {
    for check in report.checks() {
      if check.findings().is_empty() {
        continue;
      }

      let line: String = format!(
        "Check '{}' is {} with {} finding(s)",
        check.id(),
        check.status(),
        check.findings().len()
      );

      match check.status() {
        Status::Failed => xrf_output::failure!(output, "{line}"),
        Status::Error => xrf_output::error!(output, "{line}"),
        Status::Incomplete | Status::Passed | Status::Skipped => xrf_output::warning!(output, "{line}"),
      }

      for finding in check.findings() {
        xrf_output::verbose!(
          output,
          "  [{}] {}: {}",
          finding.rule_id(),
          finding.subject().unwrap_or("-"),
          finding.message()
        );
      }
    }
  }

  /// The outcome `report` means for a check of `subject`: success, a check failure counting its findings, or an
  /// execution failure where the checks reached no verdict.
  ///
  /// # Errors
  ///
  /// Returns a check failure for judged content, and an execution failure for a run that judged nothing to the end.
  pub fn conclude(output: &OutputOptions, report: &Report, subject: &str) -> CommandResult {
    match report.status() {
      Status::Passed => {
        xrf_output::success!(output, "{subject}: passed");

        Ok(())
      }
      Status::Failed => Err(CommandError::new_check_failed(report.count_findings().max(1))),
      status @ (Status::Error | Status::Incomplete | Status::Skipped) => {
        Err(XrfError::new_verify_error(format!("{subject} did not complete, status: {status}")).into())
      }
    }
  }

  /// A census's distribution as one line, `key: count` pairs in order, or none where it counted nothing: a label with
  /// nothing after it reads as a distribution that came back empty rather than a question never answered.
  pub fn format_distribution<K: Display>(counts: &BTreeMap<K, usize>) -> Option<String> {
    (!counts.is_empty()).then(|| {
      counts
        .iter()
        .map(|(key, count)| format!("{key}: {count}"))
        .collect::<Vec<String>>()
        .join(", ")
    })
  }
}
