/// What Tab makes of a line being typed: the line completed as far as the names it can still be agree, and the choices
/// left where they part, each taken to its next dot, so a dotted path is completed a group at a time.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConsoleCompletion {
  /// The line completed, with a space after a name that is complete, so its value follows.
  pub line: String,
  /// The choices left, none where one name or none is left.
  pub choices: Vec<String>,
}
