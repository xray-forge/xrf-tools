/// A variable the console shows and sets: its name, as typed, and its value as text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsoleVariable {
  pub name: String,
  pub value: String,
}
