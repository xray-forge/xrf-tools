/// One `key = value` line of a section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IniField {
  pub key: String,
  /// The value as it stands after `=`, trimmed at both ends; inside it, text is kept as written, quotes and the line
  /// breaks of a multi-line value included. Empty where only the key is written.
  pub value: String,
}

impl IniField {
  pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
    Self {
      key: key.into(),
      value: value.into(),
    }
  }

  /// Whether a value leaves a double-quoted string open, so a multi-line format reads the next line into it.
  pub fn is_quote_open(value: &str) -> bool {
    value.bytes().filter(|byte| *byte == b'"').count() % 2 == 1
  }
}
