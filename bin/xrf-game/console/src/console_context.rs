use crate::console_answer::ConsoleAnswer;
use crate::console_variable::ConsoleVariable;

/// What the console runs against: the program's own state, which owns the variables and which its commands change.
pub trait ConsoleContext {
  /// Every variable, in name order.
  fn list_variables(&self) -> Vec<ConsoleVariable>;

  /// Sets the variable `name` to `value`, answering what it now holds or why it does not take the value.
  fn set_variable(&mut self, name: &str, value: &str) -> ConsoleAnswer;
}
