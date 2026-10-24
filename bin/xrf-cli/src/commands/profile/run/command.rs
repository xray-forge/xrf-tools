use std::path::PathBuf;

use clap::builder::RangedU64ValueParser;
use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_output::OutputOptions;

use super::profile_run_binary_report::ProfileRunBinaryReport;
use super::profile_subject::ProfileSubject;
use super::report::ProfileRunReport;
use super::rounds::RoundStatistics;
use super::sample::SampledRun;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};

/// Rounds measured when the caller names no count.
///
/// Five, because three alternating rounds once showed a 10–16% difference that did not exist against a spread that
/// reaches ±1000ms on the big trees.
const DEFAULT_ROUNDS: usize = 5;

/// Rounds run and thrown away before measuring starts.
///
/// One is enough: a cold file cache costs up to twice the warm figure, and the second touch of the same tree is already
/// warm. Raising it past that buys nothing but time.
const DEFAULT_WARMUP: usize = 1;

#[derive(Default)]
pub struct RunCommand;

impl GenericCommand for RunCommand {
  fn operation(&self) -> &'static str {
    "run"
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about("Measure one invocation across builds, interleaved")
      .arg(
        Arg::new("binary")
          .help("Binary to measure; repeat to compare builds, baseline first")
          .short('b')
          .long("binary")
          .required(true)
          .num_args(1)
          .action(ArgAction::Append)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("rounds")
          .help("Measured rounds per binary")
          .long("rounds")
          .num_args(1)
          .default_value(DEFAULT_ROUNDS.to_string())
          .value_parser(RangedU64ValueParser::<usize>::new().range(1..)),
      )
      .arg(
        Arg::new("warmup")
          .help("Rounds run and discarded before measuring, so a cold file cache is not measured")
          .long("warmup")
          .num_args(1)
          .default_value(DEFAULT_WARMUP.to_string())
          .value_parser(value_parser!(usize)),
      )
      .arg(
        Arg::new("arguments")
          .help("Arguments passed to every binary, after --")
          .required(true)
          .num_args(1..)
          .last(true)
          .value_parser(value_parser!(String)),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let output: OutputOptions = context.get_output().clone();

    let binaries: Vec<PathBuf> = matches
      .get_many::<PathBuf>("binary")
      .expect("Expected at least one binary to measure")
      .cloned()
      .collect();

    let arguments: Vec<String> = matches
      .get_many::<String>("arguments")
      .expect("Expected an argument vector to measure")
      .cloned()
      .collect();

    let rounds: usize = *matches.get_one::<usize>("rounds").expect("Expected rounds to default");
    let warmup: usize = *matches.get_one::<usize>("warmup").expect("Expected warmup to default");

    let mut subjects: Vec<ProfileSubject> = binaries
      .into_iter()
      .map(|path| ProfileSubject::new(path, rounds))
      .collect::<CommandResult<Vec<ProfileSubject>>>()?;

    xrf_output::heading!(output, "Profiling {}", arguments.join(" "));
    xrf_output::info!(
      output,
      "{} binaries, {rounds} rounds after {warmup} warmup, interleaved",
      subjects.len()
    );

    // Warmup is interleaved too: a first round that ran every binary back to back would leave the last one warmest.
    for (_, index) in RoundStatistics::interleaved(subjects.len(), warmup) {
      SampledRun::measure(&subjects[index].path, &arguments)?;
    }

    for (round, index) in RoundStatistics::interleaved(subjects.len(), rounds) {
      let subject: &mut ProfileSubject = &mut subjects[index];
      let measured: SampledRun = SampledRun::measure(&subject.path, &arguments)?;

      subject.record(&measured);

      xrf_output::info!(
        output,
        "round {}/{rounds} {}: {}",
        round + 1,
        subject.label,
        xrf_utils::format_duration(measured.elapsed)
      );
    }

    let mut measured: Vec<ProfileRunBinaryReport> = subjects
      .into_iter()
      .map(|subject| {
        subject
          .into_report()
          .expect("a measured round to exist after a non-zero count")
      })
      .collect();

    // Compared against the first binary named, which is the baseline the caller chose by ordering the flags.
    if let Some(baseline) = measured.first().map(ProfileRunBinaryReport::median) {
      for entry in measured.iter_mut().skip(1) {
        entry.compare_to(baseline);
      }
    }

    for entry in &measured {
      xrf_output::info!(output, "{}", entry.describe());
    }

    context.set_result(|| ProfileRunReport::new(arguments, rounds, warmup, measured))
  }
}
