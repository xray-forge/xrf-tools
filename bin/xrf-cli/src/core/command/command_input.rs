use clap::Command;

use crate::core::arguments::InputArgument;

/// What a command reads, declared as its one positional argument so its usage line leads with the object of the verb:
/// `archive pack <path>`, never `archive pack --path <path>`. Selectors, modes and second inputs stay named flags the
/// command declares itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CommandInput {
  /// Reads no path of its own, as `docs generate` writes from the command tree alone.
  None,
  /// One required path, the help saying what it names; the command reads it back with [`InputArgument::of`].
  Path(&'static str),
}

impl CommandInput {
  /// Declares this input on `command`, before the command declares the rest of its arguments.
  pub(crate) fn declare(self, command: Command) -> Command {
    match self {
      Self::None => command,
      Self::Path(help) => InputArgument::declare(command, help),
    }
  }
}
