use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_ltx::{LtxFilesFormatter, LtxFormatOptions, LtxProjectFormatResult};
use xrf_output::OutputOptions;

use crate::commands::ltx::format::ltx_format_selection::LtxFormatSelection;
use crate::core::command_context::CommandContext;
use crate::core::command_error::CommandError;
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::progress::new_logging_job;

/// Names this many declined configs before reporting the remainder count.
const DECLINED_LIMIT: usize = 20;

#[derive(Default)]
pub struct FormatCommand;

impl GenericCommand for FormatCommand {
  fn operation(&self) -> &'static str {
    "format"
  }

  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to format ltx and ini files")
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

    let selection: LtxFormatSelection = LtxFormatSelection::select(&paths)?;

    Self::report_selection(&output, &selection, is_check, paths.len());

    let files: Vec<PathBuf> = selection.files;

    // A live sink rather than an inert job: this walks every config in a project, and a terminal owes the person
    // watching it some sign of where it has got to. Nothing here cancels.
    let options: LtxFormatOptions = LtxFormatOptions {
      output: output.clone(),
      job: new_logging_job(),
    };

    if is_check {
      let result: LtxProjectFormatResult = LtxFilesFormatter::check_format_opt(&files, options)?;

      context.set_result(|| &result)?;

      if result.invalid_files > 0 {
        return Err(CommandError::new_check_failed(result.invalid_files));
      }
    } else {
      LtxFilesFormatter::format_opt(&files, options)?;
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
