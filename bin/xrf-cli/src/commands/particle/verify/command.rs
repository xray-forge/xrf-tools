use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_db::{ParticlesFile, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};

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

  /// Create command for verifying of particle file.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to verify provided particle file")
      .arg(
        Arg::new("path")
          .help("Path to particle.xr file")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("unpacked")
          .help("Whether should verify unpacked particle")
          .short('u')
          .long("unpacked")
          .required(false)
          .required(false)
          .action(ArgAction::SetTrue),
      )
  }

  /// Verify particle file based on provided arguments.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<_>("path")
      .expect("Expected valid path to be provided");

    let unpacked: bool = matches.get_flag("unpacked");

    log::info!("Verify particle file {}, unpacked: {}", path.display(), unpacked);

    let particles_file_result: XrfResult<ParticlesFile> = if unpacked {
      ParticlesFile::import_from_path(path)
    } else {
      ParticlesFile::read_from_path::<XRayByteOrder, _>(path)
    };

    match particles_file_result {
      Ok(_) => {
        log::info!("Provided particle file is valid");

        // todo: Check nested textures.

        context.set_result(|| FileVerifyReport::passed(path))?;

        Ok(())
      }
      // An unreadable file is an execution failure; only judged content is a check failure.
      Err(error @ XrfError::Io { .. }) => Err(error.into()),
      Err(error) => {
        xrf_output::failure!(
          context.get_output().clone(),
          "Provided particle file is invalid: {error}"
        );

        // Deposited before the verdict becomes an outcome, so a caller reading `result` alone still
        // learns what was wrong.
        context.set_result(|| FileVerifyReport::failed(path, error.to_string()))?;

        Err(CommandError::new_check_failed(1))
      }
    }
  }
}
