use xrf_error::{XrfError, XrfResult};

use crate::constants::NIL;

/// How script server classes store a value that may be `nil`: `tostring` of it, a number's digits or the text `nil`.
/// A value reads as `None` for `nil`, and one `tostring` would not write is refused, since a rewrite would change it.
pub(crate) struct ScriptToString;

impl ScriptToString {
  /// Parse an id the script wrote, `None` for `nil`.
  pub fn parse_id(value: &str, what: &str) -> XrfResult<Option<u32>> {
    if value == NIL {
      return Ok(None);
    }

    match value.parse::<u32>() {
      Ok(id) if id.to_string() == value => Ok(Some(id)),
      _ => Err(XrfError::new_invalid_error(format!(
        "{what} '{value}' is neither nil nor an id"
      ))),
    }
  }

  /// The string the script writes for an id.
  pub fn format_id(value: Option<u32>) -> String {
    value.map_or_else(|| String::from(NIL), |id| id.to_string())
  }

  /// Parse text the script wrote, `None` for `nil`.
  pub fn parse_text(value: String) -> Option<String> {
    (value != NIL).then_some(value)
  }

  /// The string the script writes for text.
  pub fn format_text(value: Option<&str>) -> &str {
    value.unwrap_or(NIL)
  }
}

#[cfg(test)]
mod tests {
  use crate::data::generic::script_to_string::ScriptToString;

  #[test]
  fn reads_nil_as_none_and_writes_it_back() {
    assert_eq!(ScriptToString::parse_id("nil", "id").ok(), Some(None));
    assert_eq!(ScriptToString::format_id(None), "nil");
    assert_eq!(ScriptToString::parse_text(String::from("nil")), None);
    assert_eq!(ScriptToString::format_text(None), "nil");
  }

  #[test]
  fn reads_and_writes_a_value() {
    assert_eq!(ScriptToString::parse_id("52235", "id").ok(), Some(Some(52235)));
    assert_eq!(ScriptToString::format_id(Some(52235)), "52235");
    assert_eq!(
      ScriptToString::parse_text(String::from("walker@camp")).as_deref(),
      Some("walker@camp")
    );
  }

  /// `tostring` never writes an id this way, and reading it as one would write it back otherwise.
  #[test]
  fn refuses_an_id_it_would_not_write_back() {
    for value in ["0012", "-1", "12.5", ""] {
      let error: String = ScriptToString::parse_id(value, "Level vertex")
        .expect_err("an id the script would not write")
        .to_string();

      assert!(error.contains(&format!("Level vertex '{value}'")), "{error}");
    }
  }
}
