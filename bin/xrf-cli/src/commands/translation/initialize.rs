use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};
use xrf_output::OutputOptions;
use xrf_translation::{ProjectInitializeOptions, ProjectInitializeResult, initialize_dir, initialize_file};

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

    xrf_output::info!(output, "Verifying translation {}", path.display());

    let options: ProjectInitializeOptions = ProjectInitializeOptions {
      output,
      path: path.clone(),
    };

    let result: ProjectInitializeResult = if path.is_dir() {
      initialize_dir(path, &options)?
    } else {
      initialize_file(path, &options)?
    };

    xrf_output::info!(
      options.output,
      "Initialized translation files in {}",
      xrf_utils::format_duration(result.duration),
    );

    Ok(())
  }
}
