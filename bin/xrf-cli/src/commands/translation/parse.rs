use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};

use crate::core::command_context::CommandContext;
use crate::core::generic_command::{CommandResult, GenericCommand};

#[derive(Default)]
pub struct ParseCommand;

impl GenericCommand for ParseCommand {
  fn operation(&self) -> &'static str {
    "parse"
  }

  /// Create translation parsing command.
  fn init(&self) -> Command {
    Command::new(self.operation())
      .about("Command to parse xml translation into json variants")
      .arg(
        Arg::new("path")
          .help("Path to translation folder")
          .short('p')
          .long("path")
          .required(true)
          .value_parser(value_parser!(PathBuf)),
      )
  }

  /// Parse translation from path as json.
  fn execute(&self, _matches: &ArgMatches, _context: &mut CommandContext) -> CommandResult {
    // todo;

    Ok(())
  }
}
