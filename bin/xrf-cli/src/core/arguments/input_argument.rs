use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};

/// The positional argument a [`CommandInput`](crate::core::command::CommandInput) declares, and its reader: a command
/// states its input through `GenericCommand::input` and reads the path back here.
pub(crate) struct InputArgument;

impl InputArgument {
  pub(crate) const ID: &'static str = "input";
  const VALUE_NAME: &'static str = "path";

  /// Declares the required path on `command`, `help` saying what it names.
  pub(crate) fn declare(command: Command, help: String) -> Command {
    command.arg(
      Arg::new(Self::ID)
        .help(help)
        .value_name(Self::VALUE_NAME)
        .required(true)
        .value_parser(value_parser!(PathBuf)),
    )
  }

  /// The path the command was given.
  pub(crate) fn of(matches: &ArgMatches) -> &PathBuf {
    matches
      .get_one::<PathBuf>(Self::ID)
      .expect("Expected the required input path")
  }
}
