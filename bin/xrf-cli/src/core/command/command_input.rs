use clap::Command;

use crate::core::arguments::{InputArgument, XrayRootsArguments};

/// What a command reads, declared as its one positional argument so its usage line leads with the object of the verb:
/// `archive pack <path>`, never `archive pack --path <path>`. Selectors, modes and second inputs stay named flags the
/// command declares itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CommandInput {
  /// Reads no path of its own, as `docs generate` writes from the command tree alone.
  None,
  /// One required path, the help saying what it names; read back with [`InputArgument::of`].
  Path(String),
  /// One path the command may name its subject by instead of a named alternative, an `ArgGroup` the command declares
  /// requiring one of the two; read back with [`InputArgument::optional_of`].
  OptionalPath(String),
  /// One or more paths, each read on its own, as a formatter takes files and folders; read back with
  /// [`InputArgument::list_of`].
  Paths(String),
  /// One or more roots layered into one tree, earlier first, with `--source` and `--prefix` saying how they mount and
  /// what they are limited to; read back with [`XrayRootsArguments::of`].
  Roots {
    /// What a root holds.
    help: String,
    /// What a prefix limits the tree to.
    prefix_help: String,
  },
}

impl CommandInput {
  /// One required path, `help` saying what it names: fixed text, or text built per file format.
  pub(crate) fn path(help: impl Into<String>) -> Self {
    Self::Path(help.into())
  }

  /// One path a named alternative may stand in for, `help` saying what it names.
  pub(crate) fn optional_path(help: impl Into<String>) -> Self {
    Self::OptionalPath(help.into())
  }

  /// One or more paths read one by one, `help` saying what they name.
  pub(crate) fn paths(help: impl Into<String>) -> Self {
    Self::Paths(help.into())
  }

  /// One or more layered roots, `help` saying what a root holds and `prefix_help` what a prefix limits it to.
  pub(crate) fn roots(help: impl Into<String>, prefix_help: impl Into<String>) -> Self {
    Self::Roots {
      help: help.into(),
      prefix_help: prefix_help.into(),
    }
  }

  /// Declares this input on `command`, before the command declares the rest of its arguments.
  pub(crate) fn declare(self, command: Command) -> Command {
    match self {
      Self::None => command,
      Self::Path(help) => InputArgument::declare(command, help),
      Self::OptionalPath(help) => InputArgument::declare_optional(command, help),
      Self::Paths(help) => InputArgument::declare_many(command, help),
      Self::Roots { help, prefix_help } => XrayRootsArguments::declare(command, help, prefix_help),
    }
  }
}
