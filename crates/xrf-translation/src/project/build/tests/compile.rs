use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use xrf_utils::encode_string_to_bytes;

use crate::language::TranslationLanguage;
use crate::project::build::compile::compile_by_language;
use crate::project::build::options::ProjectBuildOptions;
use crate::types::{TranslationEntry, TranslationJson, TranslationVariant};

fn options(language: TranslationLanguage) -> ProjectBuildOptions {
  ProjectBuildOptions {
    job: Default::default(),
    is_sorted: false,
    output: xrf_output::OutputOptions::default(),
    output_dir: PathBuf::from("output"),
    language,
  }
}

fn source(id: &str, language: &str, text: &str) -> TranslationJson {
  IndexMap::from([(
    String::from(id),
    IndexMap::from([(
      String::from(language),
      Some(TranslationVariant::String(String::from(text))),
    )]),
  )])
}

#[test]
fn compiles_windows_1252_translations() {
  let compiled: String = compile_by_language(
    Path::new("translations/example.json"),
    &source("st_test", "fra", "À bientôt, José !"),
    &TranslationLanguage::French,
    &options(TranslationLanguage::French),
  )
  .unwrap();

  assert!(compiled.contains("encoding=\"windows-1252\""));
  assert!(encode_string_to_bytes(&compiled, TranslationLanguage::French.new_language_encoder()).is_ok());
}

#[test]
fn reports_unencodable_translation_entries_with_context() {
  let error = compile_by_language(
    Path::new("translations/example.json"),
    &source("st_test", "pol", "Й"),
    &TranslationLanguage::Polish,
    &options(TranslationLanguage::Polish),
  )
  .unwrap_err();

  assert_eq!(
    error.to_string(),
    "Encoding error: Translation 'translations/example.json' entry 'st_test' text cannot be encoded as windows-1250: 'Й' (U+0419)"
  );
}

#[test]
fn preserves_source_order_unless_sorting_is_requested() {
  let translations = TranslationJson::from([
    (
      String::from("st_second"),
      TranslationEntry::from([(
        String::from("eng"),
        Some(TranslationVariant::String(String::from("second"))),
      )]),
    ),
    (
      String::from("st_first"),
      TranslationEntry::from([(
        String::from("eng"),
        Some(TranslationVariant::String(String::from("first"))),
      )]),
    ),
  ]);
  let mut build_options = options(TranslationLanguage::English);

  let source_order = compile_by_language(
    Path::new("translations/example.json"),
    &translations,
    &TranslationLanguage::English,
    &build_options,
  )
  .unwrap();

  build_options.is_sorted = true;

  let sorted = compile_by_language(
    Path::new("translations/example.json"),
    &translations,
    &TranslationLanguage::English,
    &build_options,
  )
  .unwrap();

  assert!(source_order.find("st_second").unwrap() < source_order.find("st_first").unwrap());
  assert!(sorted.find("st_first").unwrap() < sorted.find("st_second").unwrap());
}

#[test]
fn a_missing_translation_compiles_to_its_own_id() {
  let compiled: String = compile_by_language(
    Path::new("translations/example.json"),
    &source("st_untranslated", "eng", "English only"),
    &TranslationLanguage::Ukrainian,
    &options(TranslationLanguage::Ukrainian),
  )
  .unwrap();

  // The engine falls back to the id too, so an untranslated string reads as its key rather than
  // vanishing from the table.
  assert!(compiled.contains("<text>st_untranslated</text>"));
}

#[test]
fn a_multi_line_entry_joins_on_the_engine_line_break() {
  let translations = TranslationJson::from([(
    String::from("st_lines"),
    TranslationEntry::from([(
      String::from("eng"),
      Some(TranslationVariant::MultiString(vec![
        String::from("first"),
        String::from("second"),
      ])),
    )]),
  )]);

  let compiled: String = compile_by_language(
    Path::new("translations/example.json"),
    &translations,
    &TranslationLanguage::English,
    &options(TranslationLanguage::English),
  )
  .unwrap();

  assert!(compiled.contains("first\\nsecond"));
}
