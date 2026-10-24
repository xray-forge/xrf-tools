use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_job::JobHandle;
use xrf_ltx::{LtxFilesFormatter, LtxFormatOptions, LtxProjectFormatResult};
use xrf_output::OutputOptions;

use crate::commands::ltx::format::ltx_format_selection::LtxFormatSelection;
use crate::commands::ltx::format::report::LtxFormatReport;
use crate::core::command::{CommandContext, CommandError, CommandResult, GenericCommand};
use crate::core::terminal::CliJob;

/// Names this many declined configs before reporting the remainder count.
const DECLINED_LIMIT: usize = 20;

#[derive(Default)]
pub struct FormatCommand;

impl GenericCommand for FormatCommand {
  fn operation(&self) -> &'static str {
    "format"
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about("Format LTX and INI files")
      .arg(
        Arg::new("path")
          .help("Paths to ltx files or folders with ltx files")
          .short('p')
          .long("path")
          .required(true)
          .num_args(1..)
          .value_parser(value_parser!(PathBuf)),
      )
      .arg(
        Arg::new("check")
          .help("Run formatter in check mode")
          .short('c')
          .long("check")
          .required(false)
          .action(ArgAction::SetTrue),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let paths: Vec<&PathBuf> = matches
      .get_many::<PathBuf>("path")
      .expect("Expected valid input paths to be provided")
      .collect();

    let is_check: bool = matches.get_flag("check");

    let output: OutputOptions = context.get_output().clone();

    // Created before the files are discovered, because its clock is what the result reports as the total: selecting
    // them over a large tree is part of the wait.
    let job: JobHandle = CliJob::new_logging();

    let selection: LtxFormatSelection = LtxFormatSelection::select(&paths)?;

    Self::report_selection(&output, &selection, is_check, paths.len());

    let options: LtxFormatOptions = LtxFormatOptions {
      output: output.clone(),
      job,
    };
    let result: LtxProjectFormatResult = if is_check {
      LtxFilesFormatter::check_format_opt(&selection.files, options)?
    } else {
      LtxFilesFormatter::format_opt(&selection.files, options)?
    };

    // Deposited in both modes, before a check's verdict becomes an outcome.
    context.set_result(|| LtxFormatReport::new(&result, &selection.declined))?;

    if is_check && result.invalid_files > 0 {
      return Err(CommandError::new_check_failed(result.invalid_files));
    }

    Ok(())
  }
}

impl FormatCommand {
  /// Reports selected-file totals and archived configs that cannot be rewritten.
  ///
  /// The first [`DECLINED_LIMIT`] declined configs are named so an installation-wide run does not look complete when
  /// archived winners were skipped.
  fn report_selection(output: &OutputOptions, selection: &LtxFormatSelection, is_check: bool, paths: usize) {
    xrf_output::info!(
      output,
      "{} {} ltx file(s) from {paths} provided path(s)",
      if is_check { "Checking" } else { "Formatting" },
      selection.files.len()
    );

    if selection.declined.is_empty() {
      return;
    }

    xrf_output::warning!(
      output,
      "Declined {} archived config(s), which cannot be rewritten in place:",
      selection.declined.len()
    );

    for declined in selection.declined.iter().take(DECLINED_LIMIT) {
      xrf_output::warning!(output, "  {declined}");
    }

    if selection.declined.len() > DECLINED_LIMIT {
      xrf_output::warning!(output, "  ... {} more", selection.declined.len() - DECLINED_LIMIT);
    }
  }
}
