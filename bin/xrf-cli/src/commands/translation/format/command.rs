use std::path::PathBuf;
use std::str::FromStr;

use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use xrf_output::OutputOptions;
use xrf_translation::{ProjectFormatOptions, ProjectFormatResult, format_sources};
use xrf_utils::LineEndings;

use crate::core::command_context::CommandContext;
use crate::core::command_error::CommandError;
use crate::core::generic_command::{CommandResult, GenericCommand};
use crate::core::progress::new_logging_job;

#[derive(Default)]
pub struct FormatCommand;

impl GenericCommand for FormatCommand {
  fn operation(&self) -> &'static str {
    "format"
  }

  /// Create translation formatting command.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to normalize json translation sources")
      .arg(
        Arg::new("path")
          .help("Paths to json translation sources or folders holding them")
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
    let paths: Vec<PathBuf> = matches
      .get_many::<PathBuf>("path")
      .expect("Expected valid input paths to be provided")
      .cloned()
      .collect();

    let is_check: bool = matches.get_flag("check");

    let line_endings: Option<LineEndings> = matches
      .get_one::<String>("line-endings")
      .map(|value: &String| LineEndings::from_str(value))
      .transpose()?;

    let output: OutputOptions = context.get_output().clone();

    let options: ProjectFormatOptions = ProjectFormatOptions {
      // Created before the sources are discovered, because its clock is what the result reports as the total:
      // selecting them over a large tree is part of the wait.
      job: new_logging_job(),
      output,
      paths,
      is_check,
      line_endings,
    };

    let result: ProjectFormatResult = format_sources(&options)?;
    let invalid_files: usize = result.invalid_files;

    context.set_result(|| &result)?;

    // Only a check judges. A rewrite that changed files did the work it was asked to do and succeeded.
    if is_check && invalid_files > 0 {
      return Err(CommandError::new_check_failed(invalid_files));
    }

    Ok(())
  }
}
