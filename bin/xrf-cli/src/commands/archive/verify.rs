use std::path::PathBuf;
use std::time::Instant;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_archive::{ArchiveFileDescriptor, ArchiveProject};
use xrf_error::XrfError;
use xrf_output::OutputOptions;

use crate::commands::archive::list::ListCommand;
use crate::core::command_error::CommandError;
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::output::TerminalOutput;

#[derive(Default)]
pub struct VerifyCommand;

impl GenericCommand for VerifyCommand {
  fn operation(&self) -> &'static str {
    "verify"
  }

  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Read every archive payload and verify decompression and CRC checks")
      .arg(
        Arg::new("path")
          .help("Path to an archive volume or a directory containing volumes")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("silent")
          .help("Disable any logging")
          .short('s')
          .long("silent")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("verbose")
          .help("Report every valid file as it is verified")
          .short('v')
          .long("verbose")
          .action(ArgAction::SetTrue),
      )
  }

  fn execute(&self, matches: &ArgMatches) -> CommandResult {
    let started_at: Instant = Instant::now();
    let path: &PathBuf = matches
      .get_one("path")
      .expect("Expected an archive path to be provided");
    let output: OutputOptions = TerminalOutput::from_options(matches.get_flag("silent"), matches.get_flag("verbose"));

    let project: ArchiveProject = ArchiveProject::new(path)?;
    let entries: Vec<&ArchiveFileDescriptor> =
      ListCommand::entries(&project, crate::commands::archive::list::ArchiveEntrySelection::Files);

    let mut findings: usize = 0;

    for entry in entries {
      match project.read_file_bytes(&entry.name) {
        Ok(_) => xrf_output::verbose!(output, "Verified {}", entry.name),
        Err(error @ XrfError::Io { .. }) => return Err(error.into()),
        Err(error) => {
          findings += 1;
          xrf_output::failure!(output, "{}: {error}", entry.name);
        }
      }
    }

    if findings == 0 {
      xrf_output::success!(
        output,
        "Verified {} file(s) in {}",
        project.files.values().filter(|entry| !entry.is_directory).count(),
        xrf_utils::format_duration(started_at.elapsed())
      );
      Ok(())
    } else {
      Err(CommandError::new_check_failed(findings))
    }
  }
}
