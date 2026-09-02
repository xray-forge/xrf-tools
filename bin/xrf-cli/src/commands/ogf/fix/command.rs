use std::path::{Path, PathBuf};

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_error::XrfError;
use xrf_output::OutputOptions;

use crate::commands::ogf::fix::ogf_fixer::OgfFixer;
use crate::commands::ogf::fix::report::OgfFixReport;
use crate::core::command_context::CommandContext;
use crate::core::execution::ExecutionArguments;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct FixCommand;

impl GenericCommand for FixCommand {
  fn operation(&self) -> &'static str {
    "fix"
  }

  /// Create command for normalizing ogf visuals.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to rewrite ogf visuals into well-formed bytes, changing nothing the engine reads")
      .arg(
        Arg::new("path")
          .help("Path to an ogf file or a directory to sweep")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dest")
          .help("Path to the resulting ogf file, defaults to in place rewrite of the source file; not for a directory")
          .short('d')
          .long("dest")
          .required(false)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("dry-run")
          .help("Report what would change and how many bytes would go without writing any file")
          .long("dry-run")
          .action(ArgAction::SetTrue),
      )
      .with_jobs()
  }

  /// Normalize the visual named, or every visual under the directory named.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid input path to be provided");
    let destination: Option<&Path> = matches.get_one::<PathBuf>("dest").map(PathBuf::as_path);
    let is_dry_run: bool = matches.get_flag("dry-run");

    let output: OutputOptions = context.get_output().clone();

    let report: OgfFixReport = OgfFixer::new(&output, is_dry_run).fix(path, destination)?;

    Self::report_summary(&output, &report);

    // Deposited before the failure becomes an outcome, so a run that could not fix every visual still reports the ones
    // it did fix and the findings explaining the rest.
    context.set_result(|| &report)?;

    if report.failed > 0 {
      return Err(
        XrfError::new_verify_error(format!(
          "Failed to fix {} of {} ogf visual(s)",
          report.failed, report.checked
        ))
        .into(),
      );
    }

    Ok(())
  }
}

impl FixCommand {
  fn report_summary(output: &OutputOptions, report: &OgfFixReport) {
    if report.is_dry_run {
      xrf_output::info!(
        output,
        "Dry run, nothing written: {} of {} visual(s) would be normalized, {} bytes discarded, {} unchanged, {} failed",
        report.normalized,
        report.checked,
        report.discarded_size,
        report.unchanged,
        report.failed
      );
    } else {
      xrf_output::info!(
        output,
        "Normalized {} of {} visual(s), {} bytes discarded, {} unchanged, {} failed",
        report.normalized,
        report.checked,
        report.discarded_size,
        report.unchanged,
        report.failed
      );
    }
  }
}
