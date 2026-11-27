use std::fmt;

use crate::console_answer::ConsoleAnswer;
use crate::console_family::ConsoleFamily;

/// What runs a command against its family's context `F::Context`, given what follows its name: a closure, so a command
/// registered while the game runs, a script's among them, carries what it needs. It takes the context for any borrow,
/// so it names the context's type in full, as `|state: &mut GameConsoleState<'_>, _|`; a closure that leaves it
/// unnamed cannot be read as one for every borrow.
pub type ConsoleRun<F> =
  Box<dyn for<'a> Fn(&mut <F as ConsoleFamily>::Context<'a>, Option<&str>) -> ConsoleAnswer + Send>;

/// A command the console runs against its family's context: its name, how it is written and what it does, for `help`
/// and the suggestions, and what runs it.
pub struct ConsoleCommand<F: ConsoleFamily> {
  pub name: String,
  /// What follows the name, as `<x> <y> <z>`; empty for a command that takes nothing.
  pub usage: String,
  pub help: String,
  pub run: ConsoleRun<F>,
}

impl<F: ConsoleFamily> ConsoleCommand<F> {
  /// The command `name`, written `name usage`, doing what `help` says by `run`.
  pub fn new(
    name: impl Into<String>,
    usage: impl Into<String>,
    help: impl Into<String>,
    run: impl for<'a> Fn(&mut F::Context<'a>, Option<&str>) -> ConsoleAnswer + Send + 'static,
  ) -> Self {
    Self {
      name: name.into(),
      usage: usage.into(),
      help: help.into(),
      run: Box::new(run),
    }
  }
}

impl<F: ConsoleFamily> fmt::Debug for ConsoleCommand<F> {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("ConsoleCommand")
      .field("name", &self.name)
      .field("usage", &self.usage)
      .field("help", &self.help)
      .finish_non_exhaustive()
  }
}
