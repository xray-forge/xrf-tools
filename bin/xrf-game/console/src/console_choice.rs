/// One name a line being typed may still become, as a suggestion shows it: the name, to its next dot for a group of
/// variables, and a hint beside it: a variable's value, a command's use, or how many variables a group holds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsoleChoice {
  pub name: String,
  pub hint: String,
}

impl ConsoleChoice {
  /// Whether it names a group of variables, `view.shadows.`, rather than one command or variable.
  pub fn is_group(&self) -> bool {
    self.name.ends_with('.')
  }
}
