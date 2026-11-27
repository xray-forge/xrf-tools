/// A line entered into the console: the name of a command or variable, then what follows it, trimmed; a list's commas
/// and spaces kept as written.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsoleLine<'a> {
  pub name: &'a str,
  /// What follows the name, none where nothing does.
  pub argument: Option<&'a str>,
}

impl<'a> ConsoleLine<'a> {
  /// The line `text` enters, none for a blank one.
  pub fn parse(text: &'a str) -> Option<Self> {
    let text: &str = text.trim();

    if text.is_empty() {
      return None;
    }

    Some(match text.split_once(char::is_whitespace) {
      Some((name, argument)) => Self {
        name,
        argument: Some(argument.trim()),
      },
      None => Self {
        name: text,
        argument: None,
      },
    })
  }
}
