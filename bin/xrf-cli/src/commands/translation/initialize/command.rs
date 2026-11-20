use std::path::PathBuf;

use clap::{ArgMatches, Command};
use xrf_output::OutputOptions;
use xrf_translation::{
  TranslationInitializeOptions, TranslationInitializeResult, TranslationInitializer, TranslationRunPrinter,
};
use xrf_utils::format_path;

use crate::core::arguments::InputArgument;
use crate::core::command::{CommandContext, CommandInput, CommandResult, GenericCommand};

#[derive(Default)]
pub(crate) struct InitializeCommand;

impl GenericCommand for InitializeCommand {
  fn operation(&self) -> &'static str {
    "initialize"
  }

  fn input(&self) -> CommandInput {
    CommandInput::path("Path to translation folder")
  }

  fn declare(&self, command: Command) -> Command {
    command.about("Initialize translation files")
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = InputArgument::of(matches);

    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(output, "Initializing translations at {}", format_path(path));

    // A file and a directory are both the operation's business: it walks one or reads the other, and the command has
    // no reason to know which it was handed.
    let result: TranslationInitializeResult = TranslationInitializer::initialize_opt(
      path,
      TranslationInitializeOptions::default().with_listener(TranslationRunPrinter::new_listener(&output)),
    )?;

    xrf_output::success!(
      output,
      "Initialized {}/{} translation source(s) in {}, {} key(s) added",
      result.files_initialized,
      result.files_read,
      xrf_utils::format_duration(result.duration),
      result.keys_added
    );

    context.set_result(|| &result)?;

    Ok(())
  }
}
