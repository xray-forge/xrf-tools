use std::fmt;

use crate::console_answer::ConsoleAnswer;

/// What runs a command against its context `C`, given what follows its name: a closure, so a command registered while
/// the game runs, a script's among them, carries what it needs.
pub type ConsoleRun<C> = Box<dyn Fn(&mut C, Option<&str>) -> ConsoleAnswer + Send>;

/// A command the console runs against its context `C`: its name, how it is written and what it does, for `help` and
/// the suggestions, and what runs it.
pub struct ConsoleCommand<C> {
  pub name: String,
  /// What follows the name, as `<x> <y> <z>`; empty for a command that takes nothing.
  pub usage: String,
  pub help: String,
  pub run: ConsoleRun<C>,
}

impl<C> ConsoleCommand<C> {
  /// The command `name`, written `name usage`, doing what `help` says by `run`.
  pub fn new(
    name: impl Into<String>,
    usage: impl Into<String>,
    help: impl Into<String>,
    run: impl Fn(&mut C, Option<&str>) -> ConsoleAnswer + Send + 'static,
  ) -> Self {
    Self {
      name: name.into(),
      usage: usage.into(),
      help: help.into(),
      run: Box::new(run),
    }
  }
}

impl<C> fmt::Debug for ConsoleCommand<C> {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("ConsoleCommand")
      .field("name", &self.name)
      .field("usage", &self.usage)
      .field("help", &self.help)
      .finish_non_exhaustive()
  }
}
