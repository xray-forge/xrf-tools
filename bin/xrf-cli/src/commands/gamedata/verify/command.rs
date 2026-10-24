use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_gamedata::{
  GamedataProject, GamedataProjectReadOptions, GamedataProjectVerifyOptions, GamedataVerificationResult,
  GamedataVerificationType,
};
use xrf_output::OutputOptions;
use xrf_utils::format_path;

use super::report::GamedataVerifyReport;
use crate::core::arguments::{EngineTargetArgument, JobsArgument, LtxDialectArgument};
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::report::ReportVerdict;
use crate::core::terminal::CliJob;

/// How many of the most-read paths `--trace-reads` names individually.
const HOTTEST_READ_PATHS_REPORTED: usize = 25;

#[derive(Default)]
pub struct VerifyCommand;

impl GenericCommand for VerifyCommand {
  fn operation(&self) -> &'static str {
    "verify"
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command
      .about("Verify an assembled gamedata tree")
      .arg(
        Arg::new("path")
          .help("Path to the assembled gamedata root")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("ignore")
          .help("Root entries to skip, replacing the defaults. Repeat to skip several")
          .short('i')
          .long("ignore")
          .action(ArgAction::Append)
          .num_args(1..)
          .default_values(GamedataProjectReadOptions::DEFAULT_IGNORED)
          .value_parser(value_parser!(String)),
      )
      .arg(
        Arg::new("checks")
          .help("List of checks to perform")
          .long("checks")
          .value_delimiter(',')
          .num_args(1..)
          .value_parser(value_parser!(GamedataVerificationType)),
      )
      .arg(
        Arg::new("strict")
          .help("Fully validate expensive asset payloads")
          .long("strict")
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("trace-reads")
          .help("Account for every asset read, reporting redundancy against unique paths")
          .long("trace-reads")
          .action(ArgAction::SetTrue),
      );

    JobsArgument::declare(EngineTargetArgument::declare(LtxDialectArgument::declare(command)))
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let root: PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected a gamedata root to be provided")
      .clone();

    let ignored: Vec<String> = matches
      .get_many::<String>("ignore")
      .map(|values| values.cloned().collect())
      .unwrap_or_default();

    let checks: Vec<GamedataVerificationType> = matches
      .get_many::<GamedataVerificationType>("checks")
      .map(|values| values.copied().collect())
      .unwrap_or_else(GamedataVerificationType::get_all);

    let output: OutputOptions = context.get_output().clone();
    let is_strict: bool = matches.get_flag("strict");

    let open_options: GamedataProjectReadOptions = GamedataProjectReadOptions {
      root: root.clone(),
      ignored,
      output: output.clone(),
      is_strict,
      is_tracing_reads: matches.get_flag("trace-reads"),
      dialect: LtxDialectArgument::of(matches),
      engine: EngineTargetArgument::of(matches),
    };

    // Created before the project opens, so the reported total covers mounting and indexing.
    let verify_options: GamedataProjectVerifyOptions = GamedataProjectVerifyOptions {
      output: output.clone(),
      is_strict,
      checks,
      job: CliJob::new_logging(),
    };

    xrf_output::heading!(output, "Opening gamedata project");
    xrf_output::info!(
      output,
      "Root: {}, ignored: [{}]",
      format_path(&open_options.root),
      open_options.ignored.join(", "),
    );

    let project: GamedataProject = GamedataProject::open(&open_options)?;

    xrf_output::info!(output, "Engine: {}", project.get_engine().describe());

    let result: GamedataVerificationResult = project.verify(&verify_options)?;

    // Deposited before the verdict is turned into an outcome, so a failing check still reports the findings.
    context.set_result(|| {
      GamedataVerifyReport::new(
        &root,
        project.get_engine(),
        &result,
        project.skipped_mounts(),
        project.get_cache_stats(),
        project.get_read_trace_summary(HOTTEST_READ_PATHS_REPORTED),
      )
    })?;

    let report: xrf_report::Report = result.to_report();

    ReportVerdict::print_findings(&output, &report);
    xrf_output::info!(
      output,
      "Gamedata project checked in {}",
      xrf_utils::format_duration(result.get_duration())
    );

    ReportVerdict::conclude(&output, &report, "Gamedata project")
  }
}
