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
    command.arg(Self::create(help))
  }

  /// Declares one or more required paths on `command`, kept in the order given, `help` saying what they name.
  pub(crate) fn declare_many(command: Command, help: String) -> Command {
    command.arg(Self::create(help).num_args(1..))
  }

  /// The path the command was given.
  pub(crate) fn of(matches: &ArgMatches) -> &PathBuf {
    matches
      .get_one::<PathBuf>(Self::ID)
      .expect("Expected the required input path")
  }

  /// The paths the command was given, in the order given.
  pub(crate) fn list_of(matches: &ArgMatches) -> Vec<&PathBuf> {
    matches
      .get_many::<PathBuf>(Self::ID)
      .expect("Expected at least one input path")
      .collect()
  }

  /// The argument both declarations share.
  fn create(help: String) -> Arg {
    Arg::new(Self::ID)
      .help(help)
      .value_name(Self::VALUE_NAME)
      .required(true)
      .value_parser(value_parser!(PathBuf))
  }
}
