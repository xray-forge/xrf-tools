use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_db::{SpawnFile, XRayByteOrder};
use xrf_error::XrfError;
use xrf_output::OutputOptions;
use xrf_utils::format_path;

use crate::core::command_context::CommandContext;
use crate::core::command_error::CommandError;
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::reports::FileVerifyReport;

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
  }

  /// Verify *.spawn file based on provided arguments.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<_>("path")
      .expect("Expected valid path to be provided");

    let output: OutputOptions = context.get_output().clone();

    log::info!("Verify spawn file {}", format_path(path));

    match SpawnFile::read_from_path::<XRayByteOrder, _>(path) {
      Ok(_) => {
        log::info!("Provided spawn file is valid");

        context.set_result(|| FileVerifyReport::passed(path))?;

        Ok(())
      }
      // An unreadable file is an execution failure; only judged content is a check failure.
      Err(error @ XrfError::Io { .. }) => Err(error.into()),
      Err(error) => {
        xrf_output::failure!(output, "Provided spawn file is invalid: {error}");

        // Deposited before the verdict becomes an outcome, so a caller reading `result` alone still
        // learns what was wrong.
        context.set_result(|| FileVerifyReport::failed(path, error.to_string()))?;

        Err(CommandError::new_check_failed(1))
      }
    }
  }
}
