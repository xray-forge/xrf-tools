use std::path::{Path, PathBuf};
use std::process::{Command as ChildCommand, Output};
use std::time::Duration;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_error::XrfError;
use xrf_output::OutputOptions;

use super::report::{ProfileReportOutput, ProfiledBinaryOutput};
use super::rounds::{RoundStatistics, interleaved};
use super::sample::{SampledRun, run_sampled};
use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

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

/// One binary as the session will measure it.
struct Subject {
  label: String,
  path: PathBuf,
  version: Vec<String>,
  runs: Vec<Duration>,
  peaks: Vec<u64>,
  means: Vec<u64>,
  exit_codes: Vec<i32>,
}

impl GenericCommand for RunCommand {
  fn operation(&self) -> &'static str {
    "run"
  }

  /// Create command to measure one command across binaries.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to measure one invocation across builds, interleaved")
      .arg(
        Arg::new("binary")
          .help("Binary to measure; repeat to compare builds, baseline first")
          .short('b')
          .long("binary")
          .required(true)
          .num_args(1)
          .action(clap::ArgAction::Append)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("rounds")
          .help("Measured rounds per binary")
          .long("rounds")
          .required(false)
          .num_args(1)
          .default_value("5")
          .value_parser(value_parser!(usize)),
      )
      .arg(
        Arg::new("warmup")
          .help("Rounds run and discarded before measuring, so a cold file cache is not measured")
          .long("warmup")
          .required(false)
          .num_args(1)
          .default_value("1")
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

    let rounds: usize = matches.get_one::<usize>("rounds").copied().unwrap_or(DEFAULT_ROUNDS);
    let warmup: usize = matches.get_one::<usize>("warmup").copied().unwrap_or(DEFAULT_WARMUP);

    if rounds == 0 {
      return Err(XrfError::new_parsing_error("Expected at least one measured round").into());
    }

    let mut subjects: Vec<Subject> = Vec::with_capacity(binaries.len());

    for path in binaries {
      if !path.is_file() {
        return Err(XrfError::new_not_found_error(format!("No binary to measure at '{}'", path.display())).into());
      }

      subjects.push(Subject {
        label: Self::label_of(&path),
        version: Self::read_version(&path)?,
        path,
        runs: Vec::with_capacity(rounds),
        peaks: Vec::with_capacity(rounds),
        means: Vec::with_capacity(rounds),
        exit_codes: Vec::new(),
      });
    }

    xrf_output::heading!(output, "Profiling {}", arguments.join(" "));
    xrf_output::info!(
      output,
      "{} binaries, {rounds} rounds after {warmup} warmup, interleaved",
      subjects.len()
    );

    // Warmup is interleaved too: a first round that ran every binary back to back would leave the last one warmest.
    for (_, index) in interleaved(subjects.len(), warmup) {
      run_sampled(&subjects[index].path, &arguments)?;
    }

    for (round, index) in interleaved(subjects.len(), rounds) {
      let subject: &mut Subject = &mut subjects[index];
      let measured: SampledRun = run_sampled(&subject.path, &arguments)?;

      subject.runs.push(measured.elapsed);

      if let Some(peak) = measured.peak_bytes {
        subject.peaks.push(peak);
      }

      if let Some(mean) = measured.mean_bytes {
        subject.means.push(mean);
      }

      if !subject.exit_codes.contains(&measured.exit_code) {
        subject.exit_codes.push(measured.exit_code);
      }

      xrf_output::info!(
        output,
        "round {}/{rounds} {}: {}",
        round + 1,
        subject.label,
        xrf_utils::format_duration(measured.elapsed)
      );
    }

    let mut measured: Vec<ProfiledBinaryOutput> = Vec::with_capacity(subjects.len());

    for subject in subjects {
      let statistics: RoundStatistics = RoundStatistics::of(&subject.runs, &subject.peaks, &subject.means)
        .expect("a measured round to exist after a non-zero count");

      measured.push(ProfiledBinaryOutput::new(
        subject.label,
        subject.path.display().to_string(),
        subject.version,
        subject.runs,
        &statistics,
        subject.exit_codes,
      ));
    }

    // Compared against the first binary named, which is the baseline the caller chose by ordering the flags.
    if let Some(baseline) = measured.first().map(|first| first.median) {
      for entry in measured.iter_mut().skip(1) {
        entry.compare_to(baseline);
      }
    }

    for entry in &measured {
      let memory: String = match (entry.peak_bytes, entry.mean_bytes) {
        (Some(peak), Some(mean)) => format!(
          ", peak {} / mean {}",
          xrf_utils::format_bytes(peak),
          xrf_utils::format_bytes(mean)
        ),
        // Absent together: both come from the same samples, and a command that finished inside one interval has none.
        _ => String::new(),
      };

      match entry.delta_percent {
        Some(delta) => xrf_output::info!(
          output,
          "{}: median {} ({delta:+.2}%){memory}",
          entry.label,
          xrf_utils::format_duration(entry.median)
        ),
        None => xrf_output::info!(
          output,
          "{}: median {}{memory}",
          entry.label,
          xrf_utils::format_duration(entry.median)
        ),
      }
    }

    context.set_result(|| ProfileReportOutput {
      command: arguments,
      rounds,
      warmup,
      binaries: measured,
    })?;

    Ok(())
  }
}

impl RunCommand {
  /// Names a binary by its file stem, disambiguated by its parent when two paths share one.
  ///
  /// Two builds of the same tool are both `xrf-cli`, and the parent directory is what a caller actually distinguished
  /// them by — `target/release` against a checkout of an older revision.
  fn label_of(path: &Path) -> String {
    let stem: String = path
      .file_stem()
      .map_or_else(|| String::from("binary"), |stem| stem.to_string_lossy().to_string());

    match path.parent().and_then(|parent| parent.file_name()) {
      Some(parent) => format!("{}/{stem}", parent.to_string_lossy()),
      None => stem,
    }
  }

  /// Asks a binary what it is, verbatim.
  ///
  /// `--version` rather than a report envelope's `build` block, so that a binary predating that field still identifies
  /// itself — which every historical revision worth measuring against does.
  fn read_version(path: &Path) -> CommandResult<Vec<String>> {
    let output: Output = ChildCommand::new(path)
      .arg("--version")
      .output()
      .map_err(|error| XrfError::new_io_error(format!("Failed to run '{}': {error}", path.display()), error.kind()))?;

    Ok(
      String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim_end().to_owned())
        .filter(|line| !line.is_empty())
        .collect(),
    )
  }
}
