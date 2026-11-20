use std::path::PathBuf;
use std::str::FromStr;

use clap::{Arg, ArgAction, ArgMatches, Command};
use xrf_output::OutputOptions;
use xrf_translation::{TranslationFormatOptions, TranslationFormatResult, TranslationFormatter, TranslationRunPrinter};
use xrf_utils::LineEndings;

use crate::core::arguments::InputArgument;
use crate::core::command::{CommandContext, CommandError, CommandInput, CommandResult, GenericCommand};
use crate::core::terminal::CliJob;

#[derive(Default)]
pub(crate) struct FormatCommand;

impl GenericCommand for FormatCommand {
  fn operation(&self) -> &'static str {
    "format"
  }

  fn input(&self) -> CommandInput {
    CommandInput::paths("Json translation sources, or folders holding them")
  }

  fn declare(&self, command: Command) -> Command {
    command
      .about("Normalize JSON translation sources")
      .arg(
        Arg::new("check")
          .help("Run formatter in check mode")
          .short('c')
          .long("check")
          .required(false)
          .action(ArgAction::SetTrue),
      )
      .arg(
        Arg::new("line-endings")
          .help("Write these line endings instead of preserving each file's own, and judge them in check mode")
          .long("line-endings")
          .value_parser(["lf", "crlf"]),
      )
  }

  /// Normalize json translation sources in place, or report which ones are not normalized.
  ///
  /// A host walk rather than a mounted tree, because this rewrites its sources and there is nowhere to put a file
  /// inside an archive volume — the same reason `translation initialize` walks the host.
  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let paths: Vec<PathBuf> = InputArgument::list_of(matches).into_iter().cloned().collect();

    let is_check: bool = matches.get_flag("check");

    let line_endings: Option<LineEndings> = matches
      .get_one::<String>("line-endings")
      .map(|value: &String| LineEndings::from_str(value))
      .transpose()?;

    let output: OutputOptions = context.get_output().clone();

    let options: TranslationFormatOptions = TranslationFormatOptions::default()
      // Created before the sources are discovered, because its clock is what the result reports as the total:
      // selecting them over a large tree is part of the wait.
      .with_job(CliJob::for_output(&output))
      .with_listener(TranslationRunPrinter::new_listener(&output))
      .with_line_endings(line_endings);

    // Two doors rather than a flag, so the call site says whether this run rewrites the tree.
    let result: TranslationFormatResult = if is_check {
      TranslationFormatter::check_format_opt(&paths, options)?
    } else {
      TranslationFormatter::format_opt(&paths, options)?
    };
    let invalid_files: usize = result.invalid_files;

    TranslationRunPrinter::print_format_result(&result, is_check, &output);

    context.set_result(|| &result)?;

    // Only a check judges. A rewrite that changed files did the work it was asked to do and succeeded.
    if is_check && invalid_files > 0 {
      return Err(CommandError::new_check_failed(invalid_files));
    }

    Ok(())
  }
}
