use std::path::PathBuf;
use std::time::Instant;

use clap::{ArgMatches, Command};
use rayon::prelude::*;
use xrf_archive::{ArchiveFileDescriptor, ArchiveProject};
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;
use xrf_report::{CheckId, CheckReport, Finding, Report, RuleId, Status};
use xrf_vfs::{XrayArchiveSource, XrayPathCollision, XrayShadowingEntry};

use super::report::ArchiveVerifyReport;
use crate::commands::archive::list::ArchiveEntrySelection;
use crate::core::arguments::{InputArgument, JobsArgument};
use crate::core::command::{CommandContext, CommandInput, CommandResult, GenericCommand};
use crate::core::report::ReportVerdict;
use crate::core::terminal::SourceReachability;

#[derive(Default)]
pub(crate) struct VerifyCommand;

impl VerifyCommand {
  /// The one check this command runs: every payload the set names reads back.
  const CHECK: &'static str = "archive.payloads";
  /// A payload that read, but not as its descriptor says: judged content.
  const RULE_PAYLOAD: &'static str = "archive.payload";
  /// A payload the volume could not hand over at all, which stops the sweep with what it judged so far, or a set
  /// whose volumes would not open, which stops it before any entry.
  const RULE_READ: &'static str = "archive.read";

  /// What reading one entry found.
  fn finding(rule: &'static str, name: &str, error: &XrfError) -> Finding {
    Finding::new(RuleId::of_static(rule), Some(String::from(name)), error.to_string())
  }

  /// Reads every entry back, judging the outcomes in the set's order.
  pub(super) fn judge(
    project: &ArchiveProject,
    entries: &[&ArchiveFileDescriptor],
    output: &OutputOptions,
  ) -> (Status, Vec<Finding>) {
    // Opened once and read by position on every worker, as an unpack reads them. Each payload drops once judged, and
    // outcomes are judged in the set's order, so the findings match a sweep one entry at a time.
    let outcomes: Vec<XrfResult> = match project.open_volumes() {
      Ok(volumes) => entries
        .par_iter()
        .map(|entry| volumes.read_bytes(entry).map(drop))
        .collect(),
      // A set that will not open has verified nothing, however few entries it names.
      Err(error) => {
        return (
          Status::Error,
          vec![Finding::new(
            RuleId::of_static(Self::RULE_READ),
            None,
            error.to_string(),
          )],
        );
      }
    };

    let mut findings: Vec<Finding> = Vec::new();

    for (entry, outcome) in entries.iter().zip(outcomes) {
      match outcome {
        Ok(_) => xrf_output::verbose!(output, "Verified {}", entry.name),
        // The sweep cannot finish, but what it already judged is still true and still worth reporting: the check
        // ends in an error carrying it, rather than a pass over the entries it never reached.
        Err(error @ XrfError::Io { .. }) => {
          findings.push(Self::finding(Self::RULE_READ, &entry.name, &error));

          return (Status::Error, findings);
        }
        Err(error) => findings.push(Self::finding(Self::RULE_PAYLOAD, &entry.name, &error)),
      }
    }

    let status: Status = if findings.is_empty() {
      Status::Passed
    } else {
      Status::Failed
    };

    (status, findings)
  }
}

impl GenericCommand for VerifyCommand {
  fn operation(&self) -> &'static str {
    "verify"
  }

  fn input(&self) -> CommandInput {
    CommandInput::path("Path to an archive volume or a directory containing volumes")
  }

  fn declare(&self, command: Command) -> Command {
    JobsArgument::declare(command.about("Read every archive payload and verify decompression and CRC checks"))
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let started_at: Instant = Instant::now();
    let path: &PathBuf = InputArgument::of(matches);
    let output: OutputOptions = context.get_output().clone();

    let project: ArchiveProject = ArchiveProject::new(path)?;
    let entries: Vec<&ArchiveFileDescriptor> = ArchiveEntrySelection::Files.select(&project);

    // Folded over the volume set this run read, rather than by mounting the path again: a second read would answer
    // over the volumes `XrayArchiveSource::read` discovers, which is not recursively the set verified here. Both
    // halves come from that one fold.
    let (overrides, unreachable): (Vec<XrayShadowingEntry>, Vec<XrayPathCollision>) =
      XrayArchiveSource::describe_overrides_of(&project);

    let checked: usize = entries.len();
    let (status, findings): (Status, Vec<Finding>) = Self::judge(&project, &entries, &output);

    let report: Report = Report::new(vec![CheckReport::new(
      CheckId::of_static(Self::CHECK),
      status,
      Some(started_at.elapsed()),
      findings,
    )]);

    SourceReachability::print_overrides(&output, &overrides);
    SourceReachability::print_collisions(&output, &unreachable);
    ReportVerdict::print_findings(&output, &report);

    let subject: String = format!("Verification of {checked} archive file(s)");

    // Deposited before the verdict becomes an outcome, so a failing check still reports what failed.
    context.set_result(|| ArchiveVerifyReport::new(checked, &overrides, unreachable, report.clone()))?;

    ReportVerdict::conclude(&output, &report, &subject)
  }
}
