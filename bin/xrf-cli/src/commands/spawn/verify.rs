use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_db::{SpawnFile, XRayByteOrder};
use xrf_error::XrfError;
use xrf_output::OutputOptions;

use crate::core::command_error::CommandError;
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::output::TerminalOutput;

#[derive(Default)]
pub struct VerifyCommand;

impl GenericCommand for VerifyCommand {
  fn operation(&self) -> &'static str {
    "verify"
  }

  /// Create command for verifying of spawn file.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to verify provided spawn file")
      .arg(
        Arg::new("path")
          .help("Path to spawn file")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("silent")
          .help("Turn off logging")
          .long("silent")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("verbose")
          .help("Turn on verbose logging")
          .short('v')
          .long("verbose")
          .required(false)
          .action(ArgAction::SetTrue),
      )
  }

  /// Verify *.spawn file based on provided arguments.
  fn execute(&self, matches: &ArgMatches) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<_>("path")
      .expect("Expected valid path to be provided");

    let output: OutputOptions = TerminalOutput::from_options(matches.get_flag("silent"), matches.get_flag("verbose"));

    log::info!("Verify spawn file {}", path.display());

    match SpawnFile::read_from_path::<XRayByteOrder, _>(path) {
      Ok(_) => {
        log::info!("Provided spawn file is valid");

        Ok(())
      }
      // An unreadable file is an execution failure; only judged content is a check failure.
      Err(error @ XrfError::Io { .. }) => Err(error.into()),
      Err(error) => {
        xrf_output::failure!(output, "Provided spawn file is invalid: {error}");

        Err(CommandError::new_check_failed(1))
      }
    }
  }
}
