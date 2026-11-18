/// One `key = value` line of a section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IniField {
  pub key: String,
  /// The text after `=`, trimmed at both ends and otherwise as written, quotes and multi-line breaks included.
  ///
  /// Empty where only the key is written.
  pub value: String,
}

impl IniField {
  pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
    Self {
      key: key.into(),
      value: value.into(),
    }
  }

  /// Whether a value leaves a double quote open, so a multi-line format reads the next line into it.
  pub fn is_quote_open(value: &str) -> bool {
    value.bytes().filter(|byte| *byte == b'"').count() % 2 == 1
  }
}
