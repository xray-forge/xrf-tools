use std::path::PathBuf;
use std::time::Instant;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_archive::{ArchiveFileDescriptor, ArchiveProject};
use xrf_error::XrfError;
use xrf_output::OutputOptions;
use xrf_report::{CheckId, CheckReport, Finding, Report, RuleId, Status};
use xrf_vfs::{XrayArchiveSource, XrayPathCollision, XrayShadowingEntry};

use super::report::ArchiveVerifyReport;
use crate::commands::archive::list::ArchiveEntrySelection;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::report::ReportVerdict;
use crate::core::terminal::SourceReachability;

#[derive(Default)]
pub struct VerifyCommand;

impl VerifyCommand {
  /// The one check this command runs: every payload the set names reads back.
  const CHECK: &'static str = "archive.payloads";
  /// A payload that read, but not as its descriptor says: judged content.
  const RULE_PAYLOAD: &'static str = "archive.payload";
  /// A payload the volume could not hand over at all, which stops the sweep with what it judged so far.
  const RULE_READ: &'static str = "archive.read";

  /// What reading one entry found.
  fn finding(rule: &'static str, name: &str, error: &XrfError) -> Finding {
    Finding::new(RuleId::of_static(rule), Some(String::from(name)), error.to_string())
  }
}

impl GenericCommand for VerifyCommand {
  fn operation(&self) -> &'static str {
    "verify"
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about("Read every archive payload and verify decompression and CRC checks")
      .arg(
        Arg::new("path")
          .help("Path to an archive volume or a directory containing volumes")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let started_at: Instant = Instant::now();
    let path: &PathBuf = matches
      .get_one("path")
      .expect("Expected an archive path to be provided");
    let output: OutputOptions = context.get_output().clone();

    let project: ArchiveProject = ArchiveProject::new(path)?;
    let entries: Vec<&ArchiveFileDescriptor> = ArchiveEntrySelection::Files.select(&project);

    // Folded over the volume set this run read, rather than by mounting the path again: a second read would answer
    // over the volumes `XrayArchiveSource::read` discovers, which is not recursively the set verified here. Both
    // halves come from that one fold.
    let (overrides, unreachable): (Vec<XrayShadowingEntry>, Vec<XrayPathCollision>) =
      XrayArchiveSource::describe_overrides_of(&project);

    let checked: usize = entries.len();
    let mut findings: Vec<Finding> = Vec::new();
    let mut status: Status = Status::Passed;

    for entry in entries {
      match project.read_file_bytes(&entry.name) {
        Ok(_) => xrf_output::verbose!(output, "Verified {}", entry.name),
        // The sweep cannot finish, but what it already judged is still true and still worth reporting: the check
        // ends in an error carrying it, rather than a pass over the entries it never reached.
        Err(error @ XrfError::Io { .. }) => {
          findings.push(Self::finding(Self::RULE_READ, &entry.name, &error));
          status = Status::Error;

          break;
        }
        Err(error) => findings.push(Self::finding(Self::RULE_PAYLOAD, &entry.name, &error)),
      }
    }

    if status == Status::Passed && !findings.is_empty() {
      status = Status::Failed;
    }

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
