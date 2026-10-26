use xrf_error::XrfResult;
use xrf_utils::LineEndings;

use crate::json::translation_json_canon::TranslationJsonCanon;
use crate::text::translation_json::TranslationJson;

fn document(source: &str) -> TranslationJson {
  serde_json::from_str(source).expect("Expected valid test JSON")
}

#[test]
fn sorts_ids_and_language_keys_naturally() {
  let mut parsed: TranslationJson = document(r#"{"st_a10":{"rus":"R","eng":"E"},"st_a2":{"eng":"E","rus":"R"}}"#);

  TranslationJsonCanon::sort(&mut parsed);

  assert_eq!(parsed.keys().collect::<Vec<_>>(), vec!["st_a2", "st_a10"]);
  assert_eq!(parsed["st_a10"].keys().collect::<Vec<_>>(), vec!["eng", "rus"]);
}

#[test]
fn renders_two_space_indentation_and_a_trailing_newline() -> XrfResult {
  let rendered: Vec<u8> =
    TranslationJsonCanon::to_bytes("test", &document(r#"{"st_a":{"eng":"A"}}"#), LineEndings::Lf)?;

  assert_eq!(
    String::from_utf8(rendered).expect("Expected UTF-8"),
    "{\n  \"st_a\": {\n    \"eng\": \"A\"\n  }\n}\n"
  );

  Ok(())
}

#[test]
fn renders_the_requested_line_endings() -> XrfResult {
  let rendered: Vec<u8> =
    TranslationJsonCanon::to_bytes("test", &document(r#"{"st_a":{"eng":"A"}}"#), LineEndings::Crlf)?;

  assert_eq!(
    String::from_utf8(rendered).expect("Expected UTF-8"),
    "{\r\n  \"st_a\": {\r\n    \"eng\": \"A\"\r\n  }\r\n}\r\n"
  );

  Ok(())
}

#[test]
fn a_null_placeholder_survives_serialization() -> XrfResult {
  let rendered: Vec<u8> =
    TranslationJsonCanon::to_bytes("test", &document(r#"{"st_a":{"eng":null}}"#), LineEndings::Lf)?;

  assert!(String::from_utf8(rendered).expect("Expected UTF-8").contains("null"));

  Ok(())
}

#[test]
fn an_explicit_choice_of_line_endings_wins_over_the_file() {
  assert_eq!(
    TranslationJsonCanon::resolve_line_endings(Some(LineEndings::Lf), Some(b"a\r\nb")),
    LineEndings::Lf
  );
  assert_eq!(
    TranslationJsonCanon::resolve_line_endings(Some(LineEndings::Crlf), Some(b"a\nb")),
    LineEndings::Crlf
  );
}

#[test]
fn an_existing_file_keeps_its_own_convention() {
  assert_eq!(
    TranslationJsonCanon::resolve_line_endings(None, Some(b"a\r\nb\r\nc")),
    LineEndings::Crlf
  );
  assert_eq!(
    TranslationJsonCanon::resolve_line_endings(None, Some(b"a\nb\nc")),
    LineEndings::Lf
  );
}

#[test]
fn a_new_file_and_a_file_with_nothing_to_read_get_lf() {
  assert_eq!(TranslationJsonCanon::resolve_line_endings(None, None), LineEndings::Lf);
  assert_eq!(
    TranslationJsonCanon::resolve_line_endings(None, Some(b"{}")),
    LineEndings::Lf
  );
}
