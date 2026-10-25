use xrf_error::{XrfError, XrfResult};
use xrf_utils::{LineEndings, natural_cmp};

use crate::types::{TranslationEntry, TranslationJson};

/// Sort ids and language keys into the order a canonical source carries them.
pub(crate) fn sort_document(document: &mut TranslationJson) {
  document.sort_by(|left, _, right, _| natural_cmp(left, right));

  for entry in document.values_mut() {
    entry.sort_by(|left, _, right, _| natural_cmp(left, right));
  }
}

/// Whether two documents would serialize identically, order included.
pub(crate) fn is_same_document(left: &TranslationJson, right: &TranslationJson) -> bool {
  left.len() == right.len()
    && left
      .iter()
      .zip(right.iter())
      .all(|((left_id, left_entry), (right_id, right_entry))| {
        left_id == right_id && is_same_entry(left_entry, right_entry)
      })
}

fn is_same_entry(left: &TranslationEntry, right: &TranslationEntry) -> bool {
  left.len() == right.len()
    && left
      .iter()
      .zip(right.iter())
      .all(|((left_language, left_value), (right_language, right_value))| {
        left_language == right_language && left_value == right_value
      })
}

/// Render a document as the bytes a canonical source holds.
///
/// # Errors
///
/// Returns a serialization error when the document cannot be rendered as JSON.
pub(crate) fn to_canonical_bytes(
  subject: &str,
  document: &TranslationJson,
  line_endings: LineEndings,
) -> XrfResult<Vec<u8>> {
  let rendered: String = serde_json::to_string_pretty(document).map_err(|error| {
    XrfError::new_serialization_error(format!("Failed to serialize translation JSON '{subject}': {error}"))
  })?;

  // Appended rather than preserved. Two of this crate's writers used to keep whatever a file already had and a third
  // silently dropped it, which is three answers to a question that has one.
  Ok(line_endings.apply(&format!("{rendered}\n")).into_bytes())
}

/// The line endings a rewrite of `existing` should use.
pub(crate) fn resolve_line_endings(requested: Option<LineEndings>, existing: Option<&[u8]>) -> LineEndings {
  requested
    .or_else(|| existing.and_then(LineEndings::detect))
    .unwrap_or(LineEndings::Lf)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn document(source: &str) -> TranslationJson {
    serde_json::from_str(source).expect("Expected valid test JSON")
  }

  #[test]
  fn sorts_ids_and_language_keys_naturally() {
    let mut parsed: TranslationJson = document(r#"{"st_a10":{"rus":"R","eng":"E"},"st_a2":{"eng":"E","rus":"R"}}"#);

    sort_document(&mut parsed);

    assert_eq!(parsed.keys().collect::<Vec<_>>(), vec!["st_a2", "st_a10"]);
    assert_eq!(parsed["st_a10"].keys().collect::<Vec<_>>(), vec!["eng", "rus"]);
  }

  #[test]
  fn two_documents_differing_only_in_order_are_not_the_same_document() {
    // The trap this function exists for: `==` answers true for both of these.
    let left: TranslationJson = document(r#"{"st_a":{"eng":"A"},"st_b":{"eng":"B"}}"#);
    let right: TranslationJson = document(r#"{"st_b":{"eng":"B"},"st_a":{"eng":"A"}}"#);

    assert_eq!(left, right);
    assert!(!is_same_document(&left, &right));
  }

  #[test]
  fn a_language_key_reordering_is_a_difference_too() {
    let left: TranslationJson = document(r#"{"st_a":{"eng":"A","rus":"R"}}"#);
    let right: TranslationJson = document(r#"{"st_a":{"rus":"R","eng":"A"}}"#);

    assert!(!is_same_document(&left, &right));
  }

  #[test]
  fn renders_two_space_indentation_and_a_trailing_newline() -> XrfResult {
    let rendered: Vec<u8> = to_canonical_bytes("test", &document(r#"{"st_a":{"eng":"A"}}"#), LineEndings::Lf)?;

    assert_eq!(
      String::from_utf8(rendered).expect("Expected UTF-8"),
      "{\n  \"st_a\": {\n    \"eng\": \"A\"\n  }\n}\n"
    );

    Ok(())
  }

  #[test]
  fn renders_the_requested_line_endings() -> XrfResult {
    let rendered: Vec<u8> = to_canonical_bytes("test", &document(r#"{"st_a":{"eng":"A"}}"#), LineEndings::Crlf)?;

    assert_eq!(
      String::from_utf8(rendered).expect("Expected UTF-8"),
      "{\r\n  \"st_a\": {\r\n    \"eng\": \"A\"\r\n  }\r\n}\r\n"
    );

    Ok(())
  }

  #[test]
  fn a_null_placeholder_survives_serialization() -> XrfResult {
    let rendered: Vec<u8> = to_canonical_bytes("test", &document(r#"{"st_a":{"eng":null}}"#), LineEndings::Lf)?;

    assert!(String::from_utf8(rendered).expect("Expected UTF-8").contains("null"));

    Ok(())
  }

  #[test]
  fn an_explicit_choice_of_line_endings_wins_over_the_file() {
    assert_eq!(
      resolve_line_endings(Some(LineEndings::Lf), Some(b"a\r\nb")),
      LineEndings::Lf
    );
    assert_eq!(
      resolve_line_endings(Some(LineEndings::Crlf), Some(b"a\nb")),
      LineEndings::Crlf
    );
  }

  #[test]
  fn an_existing_file_keeps_its_own_convention() {
    assert_eq!(resolve_line_endings(None, Some(b"a\r\nb\r\nc")), LineEndings::Crlf);
    assert_eq!(resolve_line_endings(None, Some(b"a\nb\nc")), LineEndings::Lf);
  }

  #[test]
  fn a_new_file_and_a_file_with_nothing_to_read_get_lf() {
    assert_eq!(resolve_line_endings(None, None), LineEndings::Lf);
    assert_eq!(resolve_line_endings(None, Some(b"{}")), LineEndings::Lf);
  }
}
