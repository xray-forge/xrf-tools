use std::process::ExitCode;

use crate::core::generic_command::CommandGroup;
use crate::registry::setup_command_groups;
use clap::{ArgMatches, Command};

/// Assemble the CLI from the registered commands and run the one the caller asked for.
///
/// The only place a command outcome becomes a process exit. Every failure ends with exactly one
/// final stderr line, printed unconditionally so `--silent` can never hide that a run failed;
/// commands themselves report finding details and never exit.
pub fn run() -> ExitCode {
  let groups: Vec<CommandGroup> = setup_command_groups();

  let mut application: Command = Command::new("xrf-cli")
    .about("XRF forge CLI tools application")
    .arg_required_else_help(true);

  for group in &groups {
    application = application.subcommand(group.init());
  }

  let matches: ArgMatches = application.get_matches();

  // `arg_required_else_help` already answered the empty invocation, and clap rejects a subcommand it
  // never advertised, so both misses below mean the registry and the parser disagree.
  let Some((domain, domain_matches)) = matches.subcommand() else {
    unreachable!("clap matched no subcommand after requiring one")
  };

  let Some(group) = groups.iter().find(|group| group.slug == domain) else {
    unreachable!("clap matched '{domain}', which no registered domain declares")
  };

  let Some((operation, arguments)) = domain_matches.subcommand() else {
    unreachable!("clap matched no operation after requiring one")
  };

  let Some(command) = group.commands.iter().find(|command| command.operation() == operation) else {
    unreachable!("clap matched '{domain} {operation}', which no registered command declares")
  };

  match command.execute(arguments) {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("{error}");

      ExitCode::from(error.exit_code())
    }
  }
}
