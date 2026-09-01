use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_output::OutputOptions;
use xrf_translation::{TranslationInitializeOptions, TranslationInitializeResult, TranslationInitializer};
use xrf_utils::format_path;

use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct InitializeCommand;

impl GenericCommand for InitializeCommand {
  fn operation(&self) -> &'static str {
    "initialize"
  }

  /// Create command for initialization of translation files.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to initialize translation files")
      .arg(
        Arg::new("path")
          .help("Path to translation folder")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
  }

  fn execute(&self, matches: &ArgMatches, context: &mut CommandContext) -> CommandResult {
    let path: &PathBuf = matches
      .get_one::<PathBuf>("path")
      .expect("Expected valid path to be provided");

    let output: OutputOptions = context.get_output().clone();

    xrf_output::info!(output, "Initializing translations at {}", format_path(path));

    // A file and a directory are both the operation's business: it walks one or reads the other, and the command has
    // no reason to know which it was handed.
    let result: TranslationInitializeResult = TranslationInitializer::initialize_opt(
      path,
      TranslationInitializeOptions::default().with_output(output.clone()),
    )?;

    xrf_output::info!(
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
