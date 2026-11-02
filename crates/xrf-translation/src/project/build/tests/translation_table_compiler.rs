use indexmap::IndexMap;

use crate::language::translation_language::TranslationLanguage;
use crate::project::build::compiled::translation_table_compiler::TranslationTableCompiler;
use crate::project::describe::translation_finding::TranslationFinding;
use crate::text::translation_entry::TranslationEntry;
use crate::text::translation_json::TranslationJson;
use crate::text::translation_variant::TranslationVariant;
use crate::xml::translation_line_limit::TranslationLineLimit;

const SUBJECT: &str = "translations/example.json";

fn source(id: &str, language: &str, text: &str) -> TranslationJson {
  IndexMap::from([(
    String::from(id),
    IndexMap::from([(
      String::from(language),
      Some(TranslationVariant::String(String::from(text))),
    )]),
  )])
}

fn compile(source: &TranslationJson, language: TranslationLanguage) -> String {
  let bytes: Vec<u8> = TranslationTableCompiler::compile(SUBJECT, source, language, false).unwrap();

  language.get_encoding().decode(&bytes).0.into_owned()
}

#[test]
fn compiles_in_the_code_page_the_language_is_drawn_in() {
  let bytes: Vec<u8> = TranslationTableCompiler::compile(
    SUBJECT,
    &source("st_test", "fra", "À bientôt, José !"),
    TranslationLanguage::French,
    false,
  )
  .unwrap();

  assert!(String::from_utf8_lossy(&bytes).contains("encoding=\"windows-1252\""));
  // `À` is 0xC0 in Windows-1252, the byte the `_west` fonts draw it from.
  assert!(bytes.contains(&0xC0));
}

#[test]
fn reports_unencodable_translation_entries_with_context() {
  let error = TranslationTableCompiler::compile(
    SUBJECT,
    &source("st_test", "pol", "Й"),
    TranslationLanguage::Polish,
    false,
  )
  .unwrap_err();

  assert_eq!(
    error.to_string(),
    "Encoding error: Translation 'translations/example.json' entry 'st_test' text cannot be encoded as windows-1250: 'Й' (U+0419)"
  );
}

#[test]
fn english_is_written_for_the_fonts_that_draw_it() {
  // The unprefixed fonts are Windows-1251, so a Latin accent in English would show as a Cyrillic letter.
  let error = TranslationTableCompiler::compile(
    SUBJECT,
    &source("st_test", "eng", "No way, José!"),
    TranslationLanguage::English,
    false,
  )
  .unwrap_err();

  assert!(error.to_string().contains("'é' (U+00E9)"), "{error}");
}

#[test]
fn refuses_an_entry_whose_line_the_game_would_not_load() {
  let error = TranslationTableCompiler::compile(
    SUBJECT,
    &source("st_long", "eng", &"a".repeat(TranslationLineLimit::MAX_BYTES)),
    TranslationLanguage::English,
    false,
  )
  .unwrap_err();

  assert!(error.to_string().contains("entry 'st_long' would not load"), "{error}");
}

#[test]
fn preserves_source_order_unless_sorting_is_requested() {
  let translations = TranslationJson::from([
    (
      String::from("st_10"),
      TranslationEntry::from([(
        String::from("eng"),
        Some(TranslationVariant::String(String::from("ten"))),
      )]),
    ),
    (
      String::from("st_2"),
      TranslationEntry::from([(
        String::from("eng"),
        Some(TranslationVariant::String(String::from("two"))),
      )]),
    ),
  ]);

  let source_order: String = compile(&translations, TranslationLanguage::English);
  let sorted: String = String::from_utf8(
    TranslationTableCompiler::compile(SUBJECT, &translations, TranslationLanguage::English, true).unwrap(),
  )
  .unwrap();

  assert!(source_order.find("st_10").unwrap() < source_order.find("st_2").unwrap());
  // Naturally, as a formatted source lists them, rather than by byte.
  assert!(sorted.find("st_2").unwrap() < sorted.find("st_10").unwrap());
}

#[test]
fn a_missing_translation_compiles_to_its_own_id() {
  let compiled: String = compile(
    &source("st_untranslated", "eng", "English only"),
    TranslationLanguage::Ukrainian,
  );

  // The engine falls back to the id too, so an untranslated string reads as its key rather than vanishing from the
  // table.
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

  assert!(compile(&translations, TranslationLanguage::English).contains("first\\nsecond"));
}

#[test]
fn inspecting_reports_every_problem_rather_than_the_first() {
  let mut translations: TranslationJson = source("st_accent", "eng", "José");

  translations.extend(source("st_long", "eng", &"a".repeat(TranslationLineLimit::MAX_BYTES)));
  translations.extend(source("st_fine", "eng", "Fine"));

  let findings: Vec<TranslationFinding> =
    TranslationTableCompiler::inspect(SUBJECT, &translations, TranslationLanguage::English).unwrap();
  let rules: Vec<&str> = findings.iter().map(|finding| finding.rule.as_str()).collect();

  assert_eq!(rules, vec!["translations.unencodable", "translations.overlong"]);
  assert!(findings[0].message.contains("'st_accent'"));
  assert!(findings[1].message.contains("'st_long'"));
  assert!(
    TranslationTableCompiler::inspect(SUBJECT, &source("st_fine", "eng", "Fine"), TranslationLanguage::English)
      .unwrap()
      .is_empty()
  );
}
