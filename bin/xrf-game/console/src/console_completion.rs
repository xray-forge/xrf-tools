use crate::console_choice::ConsoleChoice;

/// What a line being typed may become: the line completed as far as the names it can still be agree, as Tab completes
/// it, and every choice left, each taken to its next dot, so a dotted path is offered a group at a time.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConsoleCompletion {
  /// The line completed, with a space after a name that is whole, so its value follows.
  pub line: String,
  /// The choices left in name order, one where the line names one whole; none past the name or where none fits.
  pub choices: Vec<ConsoleChoice>,
}
