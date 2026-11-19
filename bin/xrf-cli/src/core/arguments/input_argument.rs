use std::path::PathBuf;

use clap::{Arg, ArgMatches, Command, value_parser};

/// The positional slot holding what a command reads, so its usage line leads with the object of the verb:
/// `archive pack <path>`, never `archive pack --path <path>`. A command has this one positional at most; selectors,
/// modes and second inputs stay named.
pub(crate) struct InputArgument;

impl InputArgument {
  pub(crate) const ID: &'static str = "input";
  const VALUE_NAME: &'static str = "path";

  /// Declares the required input on `command`, `help` saying what the path names.
  pub(crate) fn declare(command: Command, help: &'static str) -> Command {
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
