use std::process::ExitCode;
use std::time::{Duration, Instant};

use clap::error::ErrorKind;
use clap::{ArgMatches, Command};
use xrf_build_info::{BuildInfo, build_info};
use xrf_job::{ExecutionPlan, ExecutionRequest};

use crate::core::command_context::CommandContext;
use crate::core::command_error::CommandError;
use crate::core::execution::{report_execution_plan, requested_execution};
use crate::core::generic_command::{CommandGroup, CommandResult};
use crate::core::reporting::{CommandEnvelope, ReportingArguments, ReportingOptions, find_conflicting_selection};
use crate::registry::setup_command_groups;

/// Assemble the CLI from the registered commands and run the one the caller asked for.
///
/// The only place a command outcome becomes a process exit, and for the same reason the only place
/// the machine-readable envelope is written: an outcome and the report describing it are decided
/// together, so a run that fails still reports itself whenever JSON was asked for. Every failure
/// ends with exactly one final stderr line, printed unconditionally so `--silent` can never hide
/// that a run failed, and never on stdout so it cannot corrupt a piped document; commands themselves
/// report finding details and never exit.
pub fn run() -> ExitCode {
  let build: BuildInfo = build_info!();
  let groups: Vec<CommandGroup> = setup_command_groups();

  let mut application: Command = Command::new("xrf-cli")
    .about("XRF forge CLI tools application")
    .version(get_short_version(&build))
    .long_version(build.to_string())
    .arg_required_else_help(true)
    .with_reporting();

  for group in &groups {
    application = application.subcommand(group.init());
  }

  // Kept past parsing so a selection clap cannot refuse on its own is still refused the way clap
  // refuses one: the same message shape, the same exit code, from the same command.
  let mut parser: Command = application.clone();
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

  // Global arguments reach the operation's own matches, which is also what the command reads - and
  // is where a pair written at two different levels finally meets.
  if let Some((first, second)) = find_conflicting_selection(arguments) {
    parser
      .error(
        ErrorKind::ArgumentConflict,
        format!("the argument '{first}' cannot be used with '{second}'"),
      )
      .exit()
  }

  let reporting: ReportingOptions = ReportingOptions::from_matches(arguments);
  let mut context: CommandContext = CommandContext::new(&reporting);

  // Every command runs inside a pool, including the ones that declare no `--jobs`. A command with no
  // parallel work of its own does not stop having any: `image` reaches for Rayon underneath the DDS
  // and sprite work, and anything reaching the global pool is bounded by nothing. Installing here is
  // what makes the count an upper bound provable in one place rather than a convention every command
  // has to remember; a command that offers no say simply gets what the machine offers.
  let requested: Option<ExecutionRequest> = requested_execution(arguments);
  let plan: ExecutionPlan = requested.unwrap_or(ExecutionRequest::Auto).resolve();

  if requested.is_some() {
    report_execution_plan(&reporting.output, &plan);
  }

  let started_at: Instant = Instant::now();
  // A pool that cannot start is the command failing to run, not the command failing: it is reported
  // as an execution error and the command never sees it.
  let outcome: CommandResult = plan
    .install(|| command.execute(arguments, &mut context))
    .unwrap_or_else(|error| Err(CommandError::Execution(error)));
  let duration: Duration = started_at.elapsed();

  let envelope: CommandEnvelope = CommandEnvelope::new(
    build,
    vec![String::from(domain), String::from(operation)],
    &outcome,
    duration,
    plan,
    context.take_result(),
  );

  let result: CommandResult = match envelope.publish(&reporting.destination) {
    Ok(()) => outcome,
    Err(publication_error) => {
      if let Err(error) = outcome {
        xrf_output::error!(reporting.output, "{error}");
      }

      Err(publication_error)
    }
  };

  match result {
    Ok(()) => ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("{error}");

      ExitCode::from(error.exit_code())
    }
  }
}

/// Single-line identity: the version, how it was built, and the commit it came from.
fn get_short_version(build: &BuildInfo) -> String {
  match build.short_commit() {
    Some(commit) => format!("{} ({}, {commit})", build.version, build.kind.as_str()),
    None => format!("{} ({})", build.version, build.kind.as_str()),
  }
}
