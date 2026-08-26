use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use serde::Serialize;
use serde_json::Value;
use xrf_error::XrfError;
use xrf_output::{OutputOptions, OutputVerbosity};

use crate::core::command_error::CommandError;
use crate::core::generic_command::CommandResult;
use crate::core::output::TerminalOutput;
use crate::core::staged_write::write_file_staged;

pub const SILENT_ARGUMENT: &str = "silent";
pub const VERBOSE_ARGUMENT: &str = "verbose";
pub const JSON_ARGUMENT: &str = "json";
pub const REPORT_ARGUMENT: &str = "report";

/// Where a run's machine-readable account of itself goes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReportDestination {
  /// Nothing machine-readable was asked for, so nothing is produced.
  None,
  /// One JSON document on stdout, for a caller that pipes it.
  Stdout,
  /// One JSON document at the caller's path, written after the run.
  File(PathBuf),
}

impl ReportDestination {
  /// Reads the selection the caller made; clap has already refused an incompatible pair.
  pub fn from_matches(matches: &ArgMatches) -> Self {
    if matches.get_flag(JSON_ARGUMENT) {
      Self::Stdout
    } else {
      matches
        .get_one::<PathBuf>(REPORT_ARGUMENT)
        .map_or(Self::None, |path| Self::File(path.clone()))
    }
  }

  /// Whether a command's structured result is going to be read by anyone.
  pub const fn is_requested(&self) -> bool {
    !matches!(self, Self::None)
  }

  /// Whether stdout is still free for human-facing results.
  pub const fn is_stdout_available(&self) -> bool {
    !matches!(self, Self::Stdout)
  }
}

/// How a run ended, in the machine contract's own words.
///
/// One variant per class the exit contract distinguishes, so a consumer branches on a name instead
/// of on the numeric code. Usage errors have no variant: clap exits before a command runs, so no
/// envelope exists to carry one.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CommandOutcome {
  CheckFailed,
  ExecutionFailed,
  Success,
}

impl CommandOutcome {
  fn of(outcome: &CommandResult) -> Self {
    match outcome {
      Ok(()) => Self::Success,
      Err(CommandError::CheckFailed { .. }) => Self::CheckFailed,
      Err(CommandError::Execution(_)) => Self::ExecutionFailed,
    }
  }
}

/// One run of one command, as a machine reads it.
///
/// Deliberately small and unversioned. Everything here is something the CLI already promises through
/// its exit contract, so nothing in it can drift from what the process does; anything a command
/// wants to say beyond that lives in `result`, which is the command's own shape.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandEnvelope {
  /// The command path as it was dispatched, so a consumer never splits a string to learn it.
  command: Vec<String>,
  #[serde(with = "xrf_utils::duration_ms")]
  duration: Duration,
  error: Option<String>,
  exit_code: u8,
  outcome: CommandOutcome,
  result: Option<Value>,
}

impl CommandEnvelope {
  pub fn new(command: Vec<String>, outcome: &CommandResult, duration: Duration, result: Option<Value>) -> Self {
    Self {
      command,
      duration,
      error: outcome.as_ref().err().map(CommandError::message),
      exit_code: outcome.as_ref().err().map_or(0, CommandError::exit_code),
      outcome: CommandOutcome::of(outcome),
      result,
    }
  }

  /// Delivers the envelope, or fails saying why it could not be.
  ///
  /// A requested report is part of what the caller asked for rather than a courtesy, so failing to
  /// deliver it fails the run: a CI job that reads the destination afterwards must never take a
  /// previous run's document for this one's answer.
  pub fn publish(&self, destination: &ReportDestination) -> CommandResult {
    match destination {
      ReportDestination::None => Ok(()),
      ReportDestination::Stdout => {
        // One compact line, so a line-oriented consumer and a log both stay readable.
        let document: Vec<u8> = serde_json::to_vec(self)?;

        Self::write_to_stdout(&document).map_err(|error| {
          CommandError::from(XrfError::new_io_error(
            format!("Failed to write the report to stdout: {error}"),
            error.kind(),
          ))
        })
      }
      ReportDestination::File(path) => {
        // Trailing newline: a report is a text file, and every writer this replaced ended with one.
        let mut document: Vec<u8> = serde_json::to_vec_pretty(self)?;

        document.push(b'\n');

        write_file_staged(path, &document).map_err(|error| {
          CommandError::from(XrfError::new_io_error(
            format!("Failed to write the report to '{}': {error}", path.display()),
            error.kind(),
          ))
        })
      }
    }
  }

  fn write_to_stdout(document: &[u8]) -> io::Result<()> {
    let mut stdout: io::StdoutLock<'_> = io::stdout().lock();

    stdout.write_all(document)?;
    stdout.write_all(b"\n")?;
    stdout.flush()
  }
}

/// The output selections one run was started with.
pub struct ReportingOptions {
  pub destination: ReportDestination,
  pub output: OutputOptions,
}

impl ReportingOptions {
  /// Resolves the four reporting flags into the two things a run needs: where humans read it, and
  /// where machines do.
  pub fn from_matches(matches: &ArgMatches) -> Self {
    let destination: ReportDestination = ReportDestination::from_matches(matches);
    let verbosity: OutputVerbosity =
      get_verbosity(matches.get_flag(SILENT_ARGUMENT), matches.get_flag(VERBOSE_ARGUMENT));
    let output: OutputOptions = OutputOptions::new(
      Arc::new(TerminalOutput::new(destination.is_stdout_available())),
      verbosity,
    );

    Self { destination, output }
  }
}

/// The pair of selections that disagree, when a caller made two that cannot both hold.
///
/// `conflicts_with` is enforced within one parse level, and a global argument written at the root is
/// not the same level as one written on the operation: `xrf-cli -v archive info -s` satisfies clap
/// and would otherwise reach a run that silently discards half of what was asked for. Re-checking
/// the resolved values is what makes the rule hold wherever the flags were written.
///
/// Returned rather than raised so the composition root can answer it the way clap answers its own
/// usage errors, which is the exit code a caller already knows.
pub fn find_conflicting_selection(matches: &ArgMatches) -> Option<(&'static str, &'static str)> {
  if matches.get_flag(SILENT_ARGUMENT) && matches.get_flag(VERBOSE_ARGUMENT) {
    return Some(("--silent", "--verbose"));
  }

  if matches.get_flag(JSON_ARGUMENT) && matches.get_one::<PathBuf>(REPORT_ARGUMENT).is_some() {
    return Some(("--json", "--report <PATH>"));
  }

  None
}

/// Maps the verbosity flags a caller passed.
///
/// The contradictory pair is refused rather than resolved by precedence, so a caller is told their
/// invocation disagrees with itself instead of having one half of it silently discarded.
fn get_verbosity(is_silent: bool, is_verbose: bool) -> OutputVerbosity {
  match (is_silent, is_verbose) {
    (true, _) => OutputVerbosity::Silent,
    (false, true) => OutputVerbosity::Verbose,
    (false, false) => OutputVerbosity::Normal,
  }
}

/// Declares the reporting flags every command answers to.
///
/// Global, so they are written once and mean the same thing everywhere rather than being repeated -
/// and drifting - per command. `-s` and `-v` are theirs across the whole CLI; a command wanting a
/// short for something else picks another letter. `--json` and `--report` take none at all, since
/// the callers that reach for them are scripts, which spell flags out.
pub fn add_reporting_arguments(command: Command) -> Command {
  command
    .arg(
      Arg::new(SILENT_ARGUMENT)
        .help("Turn off logging")
        .short('s')
        .long(SILENT_ARGUMENT)
        .global(true)
        .conflicts_with(VERBOSE_ARGUMENT)
        .action(ArgAction::SetTrue),
    )
    .arg(
      Arg::new(VERBOSE_ARGUMENT)
        .help("Turn on verbose logging")
        .short('v')
        .long(VERBOSE_ARGUMENT)
        .global(true)
        .action(ArgAction::SetTrue),
    )
    .arg(
      Arg::new(JSON_ARGUMENT)
        .help("Write the run's JSON report to stdout, moving human output to stderr")
        .long(JSON_ARGUMENT)
        .global(true)
        .conflicts_with(REPORT_ARGUMENT)
        .action(ArgAction::SetTrue),
    )
    .arg(
      Arg::new(REPORT_ARGUMENT)
        .help("Write the run's JSON report to a file")
        .long(REPORT_ARGUMENT)
        .global(true)
        .value_name("PATH")
        .num_args(1)
        .value_parser(value_parser!(PathBuf)),
    )
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;
  use std::time::Duration;

  use clap::{ArgMatches, Command};
  use serde_json::{Value, json};
  use xrf_error::XrfError;
  use xrf_output::OutputVerbosity;

  use super::{
    CommandEnvelope, CommandOutcome, ReportDestination, ReportingOptions, add_reporting_arguments,
    find_conflicting_selection, get_verbosity,
  };
  use crate::core::command_error::CommandError;
  use crate::core::generic_command::CommandResult;

  fn parse(arguments: &[&str]) -> Result<ArgMatches, clap::Error> {
    add_reporting_arguments(Command::new("xrf-cli").no_binary_name(true)).try_get_matches_from(arguments)
  }

  fn envelope(outcome: CommandResult, result: Option<Value>) -> Value {
    let envelope: CommandEnvelope = CommandEnvelope::new(
      vec![String::from("archive"), String::from("info")],
      &outcome,
      Duration::from_millis(1200),
      result,
    );

    serde_json::to_value(envelope).expect("envelope to serialize")
  }

  #[test]
  fn reads_the_destination_a_caller_selected() {
    assert_eq!(
      ReportDestination::from_matches(&parse(&[]).unwrap()),
      ReportDestination::None
    );
    assert_eq!(
      ReportDestination::from_matches(&parse(&["--json"]).unwrap()),
      ReportDestination::Stdout
    );
    assert_eq!(
      ReportDestination::from_matches(&parse(&["--report", "out.json"]).unwrap()),
      ReportDestination::File(PathBuf::from("out.json"))
    );
  }

  #[test]
  fn refuses_incompatible_output_selections() {
    assert!(parse(&["--json", "--report", "out.json"]).is_err());
    assert!(parse(&["--silent", "--verbose"]).is_err());
  }

  /// The operation's own matches, from a root that carries the globals - the shape `application.rs`
  /// dispatches on, and the only place a pair written at two levels meets.
  fn parse_nested(root: &[&str], operation: &[&str]) -> ArgMatches {
    let matches: ArgMatches = add_reporting_arguments(Command::new("xrf-cli").no_binary_name(true))
      .subcommand(Command::new("info"))
      .try_get_matches_from(root.iter().chain(["info"].iter()).chain(operation.iter()))
      .expect("a pair written at two levels to satisfy clap");

    matches
      .subcommand_matches("info")
      .expect("the operation to have matched")
      .clone()
  }

  /// What clap refuses on its own never reaches here; this is the same rule for the pair it cannot
  /// see, because `conflicts_with` holds within one parse level only.
  #[test]
  fn names_a_conflicting_pair_however_it_was_written() {
    assert_eq!(find_conflicting_selection(&parse(&[]).unwrap()), None);
    assert_eq!(find_conflicting_selection(&parse(&["--silent"]).unwrap()), None);
    assert_eq!(find_conflicting_selection(&parse(&["--json"]).unwrap()), None);

    assert_eq!(
      find_conflicting_selection(&parse_nested(&["--verbose"], &["--silent"])),
      Some(("--silent", "--verbose"))
    );
    assert_eq!(
      find_conflicting_selection(&parse_nested(&["--json"], &["--report", "out.json"])),
      Some(("--json", "--report <PATH>"))
    );
    assert_eq!(find_conflicting_selection(&parse_nested(&["--verbose"], &[])), None);
  }

  #[test]
  fn leaves_stdout_to_the_document_only_in_pipe_mode() {
    assert!(ReportDestination::None.is_stdout_available());
    assert!(ReportDestination::File(PathBuf::from("out.json")).is_stdout_available());
    assert!(!ReportDestination::Stdout.is_stdout_available());
  }

  #[test]
  fn asks_for_a_structured_result_only_when_one_will_be_read() {
    assert!(!ReportDestination::None.is_requested());
    assert!(ReportDestination::Stdout.is_requested());
    assert!(ReportDestination::File(PathBuf::from("out.json")).is_requested());
  }

  #[test]
  fn maps_the_verbosity_flags() {
    assert_eq!(get_verbosity(true, false), OutputVerbosity::Silent);
    assert_eq!(get_verbosity(false, true), OutputVerbosity::Verbose);
    assert_eq!(get_verbosity(false, false), OutputVerbosity::Normal);
  }

  #[test]
  fn resolves_reporting_options_together() {
    let options: ReportingOptions = ReportingOptions::from_matches(&parse(&["--json", "--verbose"]).unwrap());

    assert_eq!(options.destination, ReportDestination::Stdout);
    assert_eq!(options.output.get_verbosity(), OutputVerbosity::Verbose);
  }

  #[test]
  fn records_a_successful_run() {
    assert_eq!(
      envelope(Ok(()), Some(json!({ "volumes": 2 }))),
      json!({
        "command": ["archive", "info"],
        "duration": 1200,
        "error": Value::Null,
        "exitCode": 0,
        "outcome": "success",
        "result": { "volumes": 2 },
      })
    );
  }

  #[test]
  fn records_a_check_failure_with_its_findings() {
    assert_eq!(
      envelope(
        Err(CommandError::new_check_failed(3)),
        Some(json!({ "status": "failed" }))
      ),
      json!({
        "command": ["archive", "info"],
        "duration": 1200,
        "error": "Check failed: 3 finding(s)",
        "exitCode": 3,
        "outcome": "checkFailed",
        "result": { "status": "failed" },
      })
    );
  }

  #[test]
  fn records_an_execution_failure_without_a_result() {
    assert_eq!(
      envelope(Err(CommandError::from(XrfError::new_generic_error("boom"))), None),
      json!({
        "command": ["archive", "info"],
        "duration": 1200,
        "error": "Generic error: boom",
        "exitCode": 1,
        "outcome": "executionFailed",
        "result": Value::Null,
      })
    );
  }

  #[test]
  fn names_every_outcome_the_exit_contract_distinguishes() {
    assert_eq!(CommandOutcome::of(&Ok(())), CommandOutcome::Success);
    assert_eq!(
      CommandOutcome::of(&Err(CommandError::new_check_failed(1))),
      CommandOutcome::CheckFailed
    );
    assert_eq!(
      CommandOutcome::of(&Err(CommandError::from(XrfError::new_generic_error("boom")))),
      CommandOutcome::ExecutionFailed
    );
  }

  #[test]
  fn publishes_nothing_when_no_report_was_asked_for() {
    let envelope: CommandEnvelope = CommandEnvelope::new(Vec::new(), &Ok(()), Duration::ZERO, None);

    assert!(envelope.publish(&ReportDestination::None).is_ok());
  }

  #[test]
  fn fails_the_run_when_a_requested_report_cannot_be_written() {
    let envelope: CommandEnvelope = CommandEnvelope::new(Vec::new(), &Ok(()), Duration::ZERO, None);
    let destination: ReportDestination = ReportDestination::File(PathBuf::from("missing-directory/report.json"));

    let error: CommandError = envelope
      .publish(&destination)
      .expect_err("an unwritable report to fail the run");

    assert_eq!(error.exit_code(), 1);
    assert!(error.message().contains("missing-directory"), "{}", error.message());
  }
}
