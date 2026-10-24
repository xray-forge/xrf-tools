use std::path::PathBuf;
use std::sync::Arc;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_output::{OutputOptions, OutputVerbosity};

use crate::core::report::{ReportDestination, ReportingOptions};
use crate::core::terminal::TerminalOutput;

/// The verbosity and machine-readable-output flags every command answers to.
///
/// Global, so they are written once on the root and mean the same thing everywhere rather than being repeated, and
/// drifting, per command. `-s` and `-v` are theirs across the whole CLI; a command wanting a short for something else
/// picks another letter. `--json` and `--report` take none at all, since the callers that reach for them are scripts,
/// which spell flags out.
pub struct ReportingArguments;

impl ReportingArguments {
  const SILENT: &'static str = "silent";
  const VERBOSE: &'static str = "verbose";
  const JSON: &'static str = "json";
  const REPORT: &'static str = "report";

  /// Declares the four on `command`, the CLI's root.
  pub fn declare(command: Command) -> Command {
    command
      .arg(
        Arg::new(Self::SILENT)
          .help("Turn off logging")
          .short('s')
          .long(Self::SILENT)
          .global(true)
          .conflicts_with(Self::VERBOSE)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new(Self::VERBOSE)
          .help("Turn on verbose logging")
          .short('v')
          .long(Self::VERBOSE)
          .global(true)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new(Self::JSON)
          .help("Write the run's JSON report to stdout, moving human output to stderr")
          .long(Self::JSON)
          .global(true)
          .conflicts_with(Self::REPORT)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new(Self::REPORT)
          .help("Write the run's JSON report to a file")
          .long(Self::REPORT)
          .global(true)
          .value_name("PATH")
          .num_args(1)
          .value_parser(value_parser!(PathBuf)),
      )
  }

  /// The two things a run needs of the four: where humans read it, and where machines do.
  pub fn of(matches: &ArgMatches) -> ReportingOptions {
    let destination: ReportDestination = Self::to_destination(matches);
    let verbosity: OutputVerbosity =
      Self::to_verbosity(matches.get_flag(Self::SILENT), matches.get_flag(Self::VERBOSE));
    let output: OutputOptions = OutputOptions::new(
      Arc::new(TerminalOutput::new(destination.is_stdout_available())),
      verbosity,
    );

    ReportingOptions { destination, output }
  }

  /// The pair of selections that disagree, when a caller made two that cannot both hold.
  ///
  /// `conflicts_with` is enforced within one parse level, and a global argument written at the root is not the same
  /// level as one written on the operation: `xrf-cli -v archive info -s` satisfies clap and would otherwise reach a run
  /// that silently discards half of what was asked for. Re-checking the resolved values is what makes the rule hold
  /// wherever the flags were written. Returned rather than raised so the composition root can answer it the way clap
  /// answers its own usage errors, which is the exit code a caller already knows.
  pub fn find_conflicting_selection(matches: &ArgMatches) -> Option<(&'static str, &'static str)> {
    if matches.get_flag(Self::SILENT) && matches.get_flag(Self::VERBOSE) {
      return Some(("--silent", "--verbose"));
    }

    if matches.get_flag(Self::JSON) && matches.get_one::<PathBuf>(Self::REPORT).is_some() {
      return Some(("--json", "--report <PATH>"));
    }

    None
  }

  /// Where the run's report goes; clap has already refused an incompatible pair.
  pub(super) fn to_destination(matches: &ArgMatches) -> ReportDestination {
    if matches.get_flag(Self::JSON) {
      ReportDestination::Stdout
    } else {
      matches
        .get_one::<PathBuf>(Self::REPORT)
        .map_or(ReportDestination::None, |path| ReportDestination::File(path.clone()))
    }
  }

  /// The verbosity the flags ask for. Silent wins; the contradictory pair never gets here, since
  /// [`Self::find_conflicting_selection`] refuses it.
  pub(super) fn to_verbosity(is_silent: bool, is_verbose: bool) -> OutputVerbosity {
    match (is_silent, is_verbose) {
      (true, _) => OutputVerbosity::Silent,
      (false, true) => OutputVerbosity::Verbose,
      (false, false) => OutputVerbosity::Normal,
    }
  }
}
