use std::fs;

use xrf_error::XrfResult;
use xrf_test_utils::utils::write_generated_test_resource;

use crate::json::read::read_json;
use crate::language::TranslationLanguage;
use crate::project::initialize::translation_initializer::TranslationInitializer;
use crate::types::TranslationVariant;

#[test]
fn initialization_replaces_json_only_after_writing_a_valid_document() -> XrfResult {
  let path = write_generated_test_resource("initialize/transactional.json", r#"{"st_test":{"eng":"original"}}"#)?;

  let result = TranslationInitializer::initialize(&path)?;

  assert_eq!(result.files_read, 1);
  assert_eq!(result.files_initialized, 1);
  assert_eq!(
    result.keys_added,
    TranslationLanguage::get_all_strings().len() as u32 - 1
  );

  let initialized = read_json(&path)?;

  assert_eq!(
    initialized["st_test"]["eng"],
    Some(TranslationVariant::String(String::from("original")))
  );
  assert!(
    TranslationLanguage::get_all_strings()
      .iter()
      .all(|language| initialized["st_test"].contains_key(language))
  );

  Ok(())
}

#[test]
fn an_already_complete_file_is_left_untouched() -> XrfResult {
  let languages: String = TranslationLanguage::get_all_strings()
    .iter()
    .map(|language| format!("\"{language}\":null"))
    .collect::<Vec<_>>()
    .join(",");
  let path = write_generated_test_resource("initialize/complete.json", format!("{{\"st_test\":{{{languages}}}}}"))?;
  let before: Vec<u8> = fs::read(&path)?;

  let result = TranslationInitializer::initialize(&path)?;

  // Nothing was added, so nothing is rewritten - a repeated run leaves the tree alone.
  assert_eq!(result.keys_added, 0);
  assert_eq!(result.files_initialized, 0);
  assert_eq!(fs::read(&path)?, before);

  Ok(())
}
