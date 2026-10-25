use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;

use crate::json::read::read_json;

#[test]
fn malformed_json_returns_a_parsing_error() -> XrfResult {
  let path = GeneratedResources::write("json_read/malformed.json", "{")?;
  let error = read_json(&path).unwrap_err();

  assert!(error.to_string().contains("Failed to parse translation JSON"));
  assert!(error.to_string().contains(&path.display().to_string()));

  Ok(())
}

#[test]
fn duplicate_json_ids_return_a_parsing_error() -> XrfResult {
  let path = GeneratedResources::write(
    "json_read/duplicate_ids.json",
    r#"{"st_test":{"eng":"first"},"st_test":{"eng":"second"}}"#,
  )?;

  // Strict here where XML is tolerant: this file is authored by the project, so a repeat is a bug.
  assert!(
    read_json(&path)
      .unwrap_err()
      .to_string()
      .contains("duplicate translation ID 'st_test'")
  );

  Ok(())
}

#[test]
fn duplicate_json_languages_return_a_parsing_error() -> XrfResult {
  let path = GeneratedResources::write(
    "json_read/duplicate_languages.json",
    r#"{"st_test":{"eng":"a","eng":"b"}}"#,
  )?;

  assert!(
    read_json(&path)
      .unwrap_err()
      .to_string()
      .contains("Duplicate translation language: 'eng'")
  );

  Ok(())
}

#[test]
fn keeps_the_order_the_file_was_authored_in() -> XrfResult {
  let path = GeneratedResources::write(
    "json_read/order.json",
    r#"{"st_second":{"eng":"b"},"st_first":{"eng":"a"}}"#,
  )?;

  assert_eq!(
    read_json(&path)?.keys().collect::<Vec<_>>(),
    vec!["st_second", "st_first"]
  );

  Ok(())
}

#[test]
fn a_null_translation_is_an_absent_one_rather_than_an_empty_string() -> XrfResult {
  let path = GeneratedResources::write("json_read/null.json", r#"{"st_test":{"eng":"a","ukr":null}}"#)?;
  let parsed = read_json(&path)?;

  assert!(parsed["st_test"].contains_key("ukr"));
  assert!(parsed["st_test"]["ukr"].is_none());

  Ok(())
}
