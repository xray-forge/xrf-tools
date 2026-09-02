use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_job::{ExecutionPlan, ExecutionRequest};
use xrf_output::OutputOptions;

pub const JOBS_ARGUMENT: &str = "jobs";

/// Declares `--jobs` on a command that has parallel work to bound.
///
/// `-j` follows every other tool that has the flag. Adding one is therefore a claim: this command's work is bounded by
/// the plan the caller gets, wherever it fans out.
pub fn add_execution_arguments(command: Command) -> Command {
  command.arg(
    Arg::new(JOBS_ARGUMENT)
      .help("How much of the machine to use: 'auto', a worker count, or a share such as '50%'")
      .short('j')
      .long(JOBS_ARGUMENT)
      .required(false)
      .value_name("JOBS")
      .num_args(1)
      .default_value("auto")
      .value_parser(value_parser!(ExecutionRequest)),
  )
}

/// What this command was asked for, or `None` where it offers no say in the matter.
///
/// Asking the parsed arguments rather than a registry, because clap already knows: a command that never declared the
/// flag cannot have matched it, so the two answers can never disagree about which commands are eligible.
pub fn requested_execution(matches: &ArgMatches) -> Option<ExecutionRequest> {
  matches
    .try_get_one::<ExecutionRequest>(JOBS_ARGUMENT)
    .ok()
    .flatten()
    .copied()
}

/// States the width this run will use, for the commands that let a caller change it.
///
/// At normal verbosity, because the number is the first thing a reproducible bug report needs and the last thing anyone
/// thinks to record. Only where `--jobs` exists: the console tells a person what they can change, and a line about a
/// knob a command does not have is noise on every other page of output.
pub fn report_execution_plan(output: &OutputOptions, plan: &ExecutionPlan) {
  xrf_output::info!(
    output,
    "Workers: {} ({})",
    plan.get_workers(),
    plan.get_origin().as_str()
  );
}

#[cfg(test)]
mod tests {
  use clap::{ArgMatches, Command};
  use xrf_job::{ExecutionOrigin, ExecutionRequest};

  use super::{add_execution_arguments, requested_execution};

  fn parse(arguments: &[&str]) -> Result<ArgMatches, clap::Error> {
    add_execution_arguments(Command::new("verify").no_binary_name(true)).try_get_matches_from(arguments)
  }

  fn parsed(arguments: &[&str]) -> Option<ExecutionRequest> {
    requested_execution(&parse(arguments).expect("the arguments parse"))
  }

  #[test]
  fn defaults_to_letting_the_machine_decide() {
    assert_eq!(parsed(&[]), Some(ExecutionRequest::Auto));
    assert_eq!(
      parsed(&[])
        .expect("a declared command always answers")
        .resolve()
        .get_origin(),
      ExecutionOrigin::Auto
    );
  }

  #[test]
  fn reads_both_spellings_of_the_flag() {
    assert_eq!(parsed(&["--jobs", "4"]), parsed(&["-j", "4"]));
    assert_eq!(parsed(&["-j", "50%"]), Some("50%".parse().expect("a share parses")));
  }

  /// A command that never declared the flag is not eligible, and says so rather than pretending to a default.
  #[test]
  fn answers_for_a_command_that_offers_no_say() {
    let matches: ArgMatches = Command::new("info")
      .no_binary_name(true)
      .try_get_matches_from::<_, &str>([])
      .expect("a command with no arguments parses");

    assert_eq!(requested_execution(&matches), None);
  }

  /// Rejection is clap's, so an unreadable value ends the run the way every other bad argument does.
  #[test]
  fn refuses_an_unreadable_value_before_the_command_runs() {
    let rejected: clap::Error = parse(&["-j", "sequential"]).expect_err("the value is refused");

    assert_eq!(rejected.kind(), clap::error::ErrorKind::ValueValidation);
    assert!(rejected.to_string().contains("auto"));
  }

  #[test]
  fn refuses_a_worker_count_nobody_meant() {
    for value in ["0", "0%", "150%", "100000"] {
      assert_eq!(
        parse(&["-j", value]).expect_err("the value is refused").kind(),
        clap::error::ErrorKind::ValueValidation,
        "{value} was accepted"
      );
    }
  }
}
