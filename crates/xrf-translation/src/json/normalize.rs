use xrf_error::{XrfError, XrfResult};
use xrf_utils::{LineEndings, apply_line_endings, detect_line_endings, natural_cmp};

use crate::types::{TranslationEntry, TranslationJson};

/// Sort ids and language keys into the order a canonical source carries them.
///
/// The whole import workflow is "run once per language and merge", so a file whose shape remembered which run touched
/// it first would churn a diff for nothing. Natural rather than byte order, matching how rustfmt orders identifiers
/// under `style_edition = "2024"` — byte order puts `st_thanks10` before `st_thanks2` and `ammo-11.43x23-fmj` before
/// `ammo-5.45x39-ap`, which reads wrong in every review that touches one.
///
/// Language keys go through the same comparator for consistency rather than for effect: they are all three lowercase
/// letters, so the two orders agree on them.
pub(crate) fn sort_document(document: &mut TranslationJson) {
  document.sort_by(|left, _, right, _| natural_cmp(left, right));

  for entry in document.values_mut() {
    entry.sort_by(|left, _, right, _| natural_cmp(left, right));
  }
}

/// Whether two documents would serialize identically, order included.
///
/// `IndexMap` compares as a map, so `==` answers true for two documents holding the same pairs in different orders —
/// which is exactly the change a re-sort makes and nothing else does. Using `==` here meant a file somebody hand-edited
/// out of order was detected as unchanged and left that way, and it is the same trap a formatter's check mode falls
/// into: every file would report as already formatted.
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
/// One serializer for every writer in this crate — the importer, the initializer, the editor and the formatter — which
/// is the only way the four of them stop disagreeing about what a source looks like. Two-space indentation, a trailing
/// newline unconditionally, and the caller's line endings.
///
/// The document is written in whatever order it already carries. Sorting is [`sort_document`]'s job, and a caller that
/// wants a canonical file calls both.
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
  Ok(apply_line_endings(&format!("{rendered}\n"), line_endings).into_bytes())
}

/// The line endings a rewrite of `existing` should use.
///
/// An explicit choice wins. Otherwise the file's own convention is preserved, because line endings are transport that
/// `.gitattributes` and `.editorconfig` own, and a formatter that flipped them would rewrite every file in a CRLF
/// checkout to say nothing about their content. A file with no line break to read, and a file that is not there yet,
/// get LF.
pub(crate) fn resolve_line_endings(requested: Option<LineEndings>, existing: Option<&[u8]>) -> LineEndings {
  requested
    .or_else(|| existing.and_then(detect_line_endings))
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
