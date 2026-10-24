use std::path::PathBuf;
use std::time::Instant;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_archive::{ArchiveFileDescriptor, ArchiveProject};
use xrf_error::XrfError;
use xrf_output::OutputOptions;
use xrf_vfs::{XrayArchiveSource, XrayPathCollision, XrayShadowingEntry};

use super::archive_verify_finding_report::ArchiveVerifyFindingReport;
use super::report::ArchiveVerifyReport;
use crate::commands::archive::list::ListCommand;
use crate::core::command::{CommandContext, CommandError, CommandResult, GenericCommand};
use crate::core::terminal::SourceReachability;

#[derive(Default)]
pub struct VerifyCommand;

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
    let entries: Vec<&ArchiveFileDescriptor> =
      ListCommand::entries(&project, crate::commands::archive::list::ArchiveEntrySelection::Files);

    // Folded over the volume set this run read, rather than by mounting the path again: a second read would answer
    // over the volumes `XrayArchiveSource::read` discovers, which is not recursively the set verified here. Both
    // halves come from that one fold.
    let (overrides, collisions): (Vec<XrayShadowingEntry>, Vec<XrayPathCollision>) =
      XrayArchiveSource::describe_overrides_of(&project);

    let checked: usize = entries.len();
    let mut findings: Vec<ArchiveVerifyFindingReport> = Vec::new();

    for entry in entries {
      match project.read_file_bytes(&entry.name) {
        Ok(_) => xrf_output::verbose!(output, "Verified {}", entry.name),
        // The sweep cannot finish, but what it already judged is still true and still worth
        // reporting: depositing before giving up keeps the entries found corrupt out of the run's
        // loss column.
        Err(error @ XrfError::Io { .. }) => {
          context.set_result(|| ArchiveVerifyReport::new(checked, &overrides, collisions, findings))?;

          return Err(error.into());
        }
        Err(error) => {
          xrf_output::failure!(output, "{}: {error}", entry.name);
          findings.push(ArchiveVerifyFindingReport::new(&entry.name, error.to_string()));
        }
      }
    }

    let finding_count: usize = findings.len();

    SourceReachability::print_overrides(&output, &overrides);
    SourceReachability::print_collisions(&output, &collisions);

    // Deposited before the verdict becomes an outcome, so a failing check still reports what failed.
    context.set_result(|| ArchiveVerifyReport::new(checked, &overrides, collisions, findings))?;

    if finding_count == 0 {
      xrf_output::success!(
        output,
        "Verified {} file(s) in {}",
        checked,
        xrf_utils::format_duration(started_at.elapsed())
      );
      Ok(())
    } else {
      Err(CommandError::new_check_failed(finding_count))
    }
  }
}
