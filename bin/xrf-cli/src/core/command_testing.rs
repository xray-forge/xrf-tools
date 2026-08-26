//! Test-only helpers for driving a command the way the process drives it.

use clap::ArgMatches;
use serde_json::Value;

use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::reporting::{ReportingOptions, add_reporting_arguments};

/// Parses `arguments` and runs `command` over them exactly as `application::run` would.
///
/// Assembling the command the same way production does is the point: the reporting flags live on the
/// root, so a test that built the bare subcommand would neither accept `--silent` nor resolve the
/// output a command writes through.
pub fn run_command<T: GenericCommand>(command: &T, arguments: &[String]) -> CommandResult {
  run_command_for_result(command, arguments).map(|_| ())
}

/// Runs a command and returns the structured result it deposited.
///
/// Publishing that result is the composition root's job and is pinned end to end, so a command test
/// asks what the command reported rather than reading a file. Pass `--json` or `--report` in
/// `arguments`: without one, nothing asked for a result and the command is right to skip building it.
pub fn run_command_for_result<T: GenericCommand>(command: &T, arguments: &[String]) -> CommandResult<Option<Value>> {
  let matches: ArgMatches = add_reporting_arguments(command.init()).try_get_matches_from(arguments)?;
  let mut context: CommandContext = CommandContext::new(&ReportingOptions::from_matches(&matches));

  command.execute(&matches, &mut context)?;

  Ok(context.take_result())
}
