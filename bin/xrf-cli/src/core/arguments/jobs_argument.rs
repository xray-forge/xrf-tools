use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_job::{ExecutionPlan, ExecutionRequest};
use xrf_output::OutputOptions;

use crate::core::arguments::OptionalArgument;

/// `--jobs`, on a command that has parallel work to bound.
///
/// Per command rather than global, unlike the reporting flags. Those describe the run itself and mean the same thing
/// everywhere; this one describes work, and a command with none has nothing to say about it. The generated reference is
/// published, so a flag advertised on every page and honoured on three would be a documentation defect at scale rather
/// than a harmless convenience.
pub struct JobsArgument;

impl JobsArgument {
  const NAME: &'static str = "jobs";

  /// Declares `--jobs` on `command`. `-j` follows every other tool that has the flag; declaring it is a claim that the
  /// command's work is bounded by the plan the caller gets, wherever it fans out.
  pub fn declare(command: Command) -> Command {
    command.arg(
      Arg::new(Self::NAME)
        .help("How much of the machine to use: 'auto', a worker count, or a share such as '50%'")
        .short('j')
        .long(Self::NAME)
        .required(false)
        .value_name("JOBS")
        .num_args(1)
        .default_value("auto")
        .value_parser(value_parser!(ExecutionRequest)),
    )
  }

  /// What the command was asked for, or `None` where it offers no say in the matter.
  pub fn of(matches: &ArgMatches) -> Option<ExecutionRequest> {
    OptionalArgument::read::<ExecutionRequest>(matches, Self::NAME).copied()
  }

  /// States the width this run will use, for the commands that let a caller change it.
  ///
  /// At normal verbosity, because the number is the first thing a reproducible bug report needs and the last thing
  /// anyone thinks to record. Only where `--jobs` exists: the console tells a person what they can change, and a line
  /// about a knob a command does not have is noise on every other page of output.
  pub fn report(output: &OutputOptions, plan: &ExecutionPlan) {
    xrf_output::info!(
      output,
      "Workers: {} ({})",
      plan.get_workers(),
      plan.get_origin().as_str()
    );
  }
}
