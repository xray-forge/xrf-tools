use std::marker::PhantomData;
use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command};
use xrf_error::{XrfError, XrfResult};
use xrf_output::OutputOptions;
use xrf_utils::format_path;

use crate::commands::packed_file::{PackedFileArguments, PackedFileFormat};
use crate::core::command::{CommandContext, CommandError, CommandResult, GenericCommand};
use crate::core::reports::FileVerifyReport;

/// Reads one packed file, or its unpacked folder, and judges whether it reads.
pub struct VerifyCommand<F>(PhantomData<F>);

impl<F> Default for VerifyCommand<F> {
  fn default() -> Self {
    Self(PhantomData)
  }
}

impl<F> VerifyCommand<F> {
  const UNPACKED: &'static str = "unpacked";
}

impl<F: PackedFileFormat> GenericCommand for VerifyCommand<F> {
  fn operation(&self) -> &'static str {
    "verify"
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command.about(format!("Verify that a {} reads", F::FILE_NAME));

    PackedFileArguments::<F>::declare_path(
      command,
      format!("Packed {}, or its unpacked folder with --unpacked", F::FILE_NAME),
    )
    .arg(
      Arg::new(Self::UNPACKED)
        .help(format!("Read the path as an unpacked {} folder", F::NOUN))
        .short('u')
        .long(Self::UNPACKED)
        .action(ArgAction::SetTrue),
    )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = PackedFileArguments::<F>::path_of(matches);
    let is_unpacked: bool = matches.get_flag(Self::UNPACKED);
    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(output, "Verifying {} {}", F::NOUN, format_path(path));

    let read: XrfResult<F> = if is_unpacked { F::import(path) } else { F::read(path) };

    match read {
      Ok(_) => {
        xrf_output::success!(output, "{} is a valid {}", format_path(path), F::NOUN);

        context.set_result(|| FileVerifyReport::passed(path))
      }
      // An unreadable file is an execution failure; only judged content is a check failure.
      Err(error @ XrfError::Io { .. }) => Err(error.into()),
      Err(error) => {
        xrf_output::failure!(output, "{} is not a valid {}: {error}", format_path(path), F::NOUN);

        // Deposited before the verdict becomes an outcome, so a caller reading `result` alone still learns what was
        // wrong.
        context.set_result(|| FileVerifyReport::failed(path, error.to_string()))?;

        Err(CommandError::new_check_failed(1))
      }
    }
  }
}
