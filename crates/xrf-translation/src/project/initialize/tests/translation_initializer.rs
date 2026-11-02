use std::fs;

use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;

use crate::json::translation_json_reader::TranslationJsonReader;
use crate::language::translation_language::TranslationLanguage;
use crate::project::initialize::translation_initializer::TranslationInitializer;
use crate::text::translation_variant::TranslationVariant;

#[test]
fn initialization_replaces_json_only_after_writing_a_valid_document() -> XrfResult {
  let path = GeneratedResources::write("initialize/transactional.json", r#"{"st_test":{"eng":"original"}}"#)?;

  let result = TranslationInitializer::initialize(&path)?;

  assert_eq!(result.files_read, 1);
  assert_eq!(result.files_initialized, 1);
  assert_eq!(result.keys_added, TranslationLanguage::LANGUAGES.len() as u32 - 1);

  let initialized = TranslationJsonReader::read(&path)?;

  assert_eq!(
    initialized["st_test"]["eng"],
    Some(TranslationVariant::String(String::from("original")))
  );
  assert!(
    TranslationLanguage::LANGUAGES
      .iter()
      .all(|language| initialized["st_test"].contains_key(&language.to_string()))
  );

  Ok(())
}

#[test]
fn an_already_complete_file_is_left_untouched() -> XrfResult {
  let languages: String = TranslationLanguage::LANGUAGES
    .iter()
    .map(|language| format!("\"{language}\":null"))
    .collect::<Vec<_>>()
    .join(",");
  let path = GeneratedResources::write("initialize/complete.json", format!("{{\"st_test\":{{{languages}}}}}"))?;
  let before: Vec<u8> = fs::read(&path)?;

  let result = TranslationInitializer::initialize(&path)?;

  // Nothing was added, so nothing is rewritten - a repeated run leaves the tree alone.
  assert_eq!(result.keys_added, 0);
  assert_eq!(result.files_initialized, 0);
  assert_eq!(fs::read(&path)?, before);

  Ok(())
}
