use std::str::FromStr;

use xrf_utils::{new_windows1250_encoder, new_windows1252_encoder};

use crate::TranslationLanguage;
use crate::language::find_unencodable_character;

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
fn selects_the_xray_encoding_for_each_language() {
  assert_eq!(TranslationLanguage::English.get_language_encoding(), "windows-1252");
  assert_eq!(TranslationLanguage::French.get_language_encoding(), "windows-1252");
  assert_eq!(TranslationLanguage::Italian.get_language_encoding(), "windows-1252");
  assert_eq!(TranslationLanguage::Spanish.get_language_encoding(), "windows-1252");
  assert_eq!(TranslationLanguage::German.get_language_encoding(), "windows-1250");
  assert_eq!(TranslationLanguage::Polish.get_language_encoding(), "windows-1250");
  assert_eq!(TranslationLanguage::Russian.get_language_encoding(), "windows-1251");
  assert_eq!(TranslationLanguage::Ukrainian.get_language_encoding(), "windows-1251");
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
fn finds_the_character_an_encoding_cannot_hold() {
  assert_eq!(
    find_unencodable_character("plain ascii", new_windows1252_encoder()),
    None
  );
  assert_eq!(
    find_unencodable_character("Привет", new_windows1252_encoder()),
    Some('П')
  );
  // Cyrillic is not in 1250 either, which is what makes a Polish target refuse it.
  assert_eq!(find_unencodable_character("Й", new_windows1250_encoder()), Some('Й'));
}
