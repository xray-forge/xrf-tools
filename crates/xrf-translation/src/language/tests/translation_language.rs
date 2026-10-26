use std::str::FromStr;

use crate::TranslationLanguage;

#[test]
fn test_from_str() {
  assert_eq!(
    TranslationLanguage::from_str("eng").unwrap(),
    TranslationLanguage::English
  );
  assert_eq!(
    TranslationLanguage::from_str("ukr").unwrap(),
    TranslationLanguage::Ukrainian
  );
  assert_eq!(TranslationLanguage::from_str("all").unwrap(), TranslationLanguage::All);
}

#[test]
fn test_from_str_single() {
  assert!(TranslationLanguage::from_str_single("all").is_err());
  assert_eq!(
    TranslationLanguage::from_str_single("eng").unwrap(),
    TranslationLanguage::English
  );
  assert_eq!(
    TranslationLanguage::from_str_single("spa").unwrap(),
    TranslationLanguage::Spanish
  );
}

#[test]
fn selects_the_code_page_the_game_draws_each_language_in() {
  // The `_west` fonts.
  assert_eq!(TranslationLanguage::French.get_encoding_label(), "windows-1252");
  assert_eq!(TranslationLanguage::German.get_encoding_label(), "windows-1252");
  assert_eq!(TranslationLanguage::Italian.get_encoding_label(), "windows-1252");
  assert_eq!(TranslationLanguage::Spanish.get_encoding_label(), "windows-1252");
  // The `_cent` fonts.
  assert_eq!(TranslationLanguage::Polish.get_encoding_label(), "windows-1250");
  // The unprefixed fonts, which English is drawn with too.
  assert_eq!(TranslationLanguage::English.get_encoding_label(), "windows-1251");
  assert_eq!(TranslationLanguage::Russian.get_encoding_label(), "windows-1251");
  assert_eq!(TranslationLanguage::Ukrainian.get_encoding_label(), "windows-1251");
}

#[test]
fn the_encoder_and_its_label_name_the_same_code_page() {
  for language in TranslationLanguage::LANGUAGES {
    assert_eq!(
      language.get_encoding().name().to_lowercase(),
      language.get_encoding_label(),
      "{language}"
    );
  }
}

#[test]
fn reads_the_language_off_a_gamedata_directory() {
  assert_eq!(
    TranslationLanguage::from_directory_name("rus"),
    Some(TranslationLanguage::Russian)
  );
  assert_eq!(TranslationLanguage::from_directory_name("cze"), None);
}

#[test]
fn a_language_crosses_the_wire_as_the_code_it_parses_from() {
  for language in TranslationLanguage::LANGUAGES
    .into_iter()
    .chain([TranslationLanguage::All])
  {
    let wire: String = serde_json::to_string(&language).expect("serialized");

    assert_eq!(wire, format!("\"{language}\""));
    assert_eq!(
      serde_json::from_str::<TranslationLanguage>(&wire).expect("deserialized"),
      language
    );
  }
}

#[test]
fn only_all_is_refused_as_a_single_language() {
  assert!(TranslationLanguage::All.into_single().is_err());
  assert_eq!(
    TranslationLanguage::Polish.into_single().expect("a single language"),
    TranslationLanguage::Polish
  );
}
