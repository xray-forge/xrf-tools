use std::fs;
use xrf_error::XrfResult;
use xrf_test_utils::utils::write_generated_test_resource;

use crate::edit::TranslationEdit;
use crate::json::read::read_json;
use crate::json::write::apply_edits;
use crate::types::{TranslationEntry, TranslationJson, TranslationVariant};

fn set(id: &str, text: &str) -> TranslationEdit {
  TranslationEdit::Set {
    id: String::from(id),
    value: TranslationVariant::String(String::from(text)),
  }
}

fn remove(id: &str) -> TranslationEdit {
  TranslationEdit::Remove { id: String::from(id) }
}

#[test]
fn sets_one_language_and_leaves_the_others() -> XrfResult {
  let path = write_generated_test_resource(
    "json_write/set.json",
    "{\n  \"st_hello\": {\n    \"eng\": \"Hello\",\n    \"ukr\": \"Pryvit\"\n  }\n}\n",
  )?;

  apply_edits(&path, "ukr", &[set("st_hello", "Vitayu")])?;

  let parsed: TranslationJson = read_json(&path)?;
  let entry: &TranslationEntry = &parsed["st_hello"];

  assert_eq!(
    entry["eng"].as_ref(),
    Some(&TranslationVariant::String(String::from("Hello")))
  );
  assert_eq!(
    entry["ukr"].as_ref(),
    Some(&TranslationVariant::String(String::from("Vitayu")))
  );

  Ok(())
}

#[test]
fn adds_an_entry_that_was_not_there() -> XrfResult {
  let path = write_generated_test_resource("json_write/add.json", "{\n  \"st_a\": {\n    \"eng\": \"A\"\n  }\n}\n")?;

  apply_edits(&path, "eng", &[set("st_b", "B")])?;

  // Appended, so the order the file was authored in survives a save.
  assert_eq!(read_json(&path)?.keys().collect::<Vec<_>>(), vec!["st_a", "st_b"]);

  Ok(())
}

#[test]
fn removing_a_language_keeps_the_entry_for_the_rest() -> XrfResult {
  let path = write_generated_test_resource(
    "json_write/remove_one.json",
    "{\n  \"st_hello\": {\n    \"eng\": \"Hello\",\n    \"ukr\": \"Pryvit\"\n  }\n}\n",
  )?;

  apply_edits(&path, "ukr", &[remove("st_hello")])?;

  let parsed: TranslationJson = read_json(&path)?;

  assert!(parsed["st_hello"].contains_key("eng"));
  assert!(!parsed["st_hello"].contains_key("ukr"));

  Ok(())
}

#[test]
fn an_entry_with_no_languages_left_goes_entirely() -> XrfResult {
  let path = write_generated_test_resource(
    "json_write/remove_last.json",
    "{\n  \"st_only\": {\n    \"eng\": \"Only\"\n  }\n}\n",
  )?;

  apply_edits(&path, "eng", &[remove("st_only")])?;

  assert!(read_json(&path)?.is_empty());

  Ok(())
}

#[test]
fn a_file_without_a_trailing_newline_gains_one() -> XrfResult {
  // Every writer in this crate goes through the canonical serializer, which appends one unconditionally. Preserving
  // its absence was one of three answers the crate used to give: the importer and this editor kept whatever a file
  // had, and the initializer silently dropped it.
  let path = write_generated_test_resource(
    "json_write/no_newline.json",
    "{\n  \"st_a\": {\n    \"eng\": \"A\"\n  }\n}",
  )?;

  apply_edits(&path, "eng", &[set("st_a", "B")])?;

  assert!(fs::read(&path)?.ends_with(b"\n"));

  Ok(())
}

#[test]
fn a_file_with_a_trailing_newline_keeps_it() -> XrfResult {
  let path = write_generated_test_resource(
    "json_write/newline.json",
    "{\n  \"st_a\": {\n    \"eng\": \"A\"\n  }\n}\n",
  )?;

  apply_edits(&path, "eng", &[set("st_a", "B")])?;

  assert!(fs::read(&path)?.ends_with(b"\n"));

  Ok(())
}
