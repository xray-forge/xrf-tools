use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_job::JobHandle;
use xrf_ltx::{LtxProject, LtxProjectVerifyResult, LtxVerifyOptions};
use xrf_output::OutputOptions;
use xrf_utils::format_path;

use crate::commands::ltx::ltx_installation::LtxInstallation;
use crate::commands::ltx::ltx_project_opening::LtxProjectOpening;
use crate::commands::ltx::verify::report::LtxVerifyReport;
use crate::core::arguments::LtxDialectArgument;
use crate::core::command::{CommandContext, CommandResult, GenericCommand};
use crate::core::report::ReportVerdict;
use crate::core::terminal::CliJob;

#[derive(Default)]
pub struct VerifyCommand;

impl GenericCommand for VerifyCommand {
  fn operation(&self) -> &'static str {
    "verify"
  }

  fn declare(&self, command: Command) -> Command {
    let command: Command = command.about("Verify LTX and INI configs").arg(
      Arg::new("path")
        .help(LtxInstallation::PATH_HELP)
        .short('p')
        .long("path")
        .required(true)
        .value_parser(value_parser!(PathBuf)),
    );

    LtxDialectArgument::declare(command)
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid input path to be provided");

    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(output, "Verifying LTX configs in {}", format_path(path));

    // Created before the project is opened, because its clock is what the result reports as the total: mounting and
    // indexing a cold installation is most of the wait.
    let job: JobHandle = CliJob::for_output(&output);

    // Verification only reads, so an installation is verified over every declared source. Narrowing it to loose configs
    // would leave archived includes unresolved and report their sections as missing, so the scope a person picks is the
    // path they name rather than a flag.
    let project: LtxProject = LtxProjectOpening::open(path, matches, &output)?;
    let result: LtxProjectVerifyResult = project.verify_entries_opt(LtxVerifyOptions {
      output: output.clone(),
      job,
    })?;
    let report: LtxVerifyReport = LtxVerifyReport::new(&project, &result);

    ReportVerdict::print_findings(&output, report.report());

    // Deposited before the verdict becomes an outcome, so a failing check still reports what failed.
    context.set_result(|| &report)?;

    ReportVerdict::conclude(&output, report.report(), "LTX verification")
  }
}
