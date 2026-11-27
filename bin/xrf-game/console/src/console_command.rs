use crate::console_answer::ConsoleAnswer;

/// A command the console runs against its context `C`: its name, how it is written and what it does, for `help`, and
/// what runs it, given what follows its name.
pub struct ConsoleCommand<C> {
  pub name: &'static str,
  /// What follows the name, as `<x> <y> <z>`; empty for a command that takes nothing.
  pub usage: &'static str,
  pub help: &'static str,
  pub run: fn(&mut C, Option<&str>) -> ConsoleAnswer,
}
