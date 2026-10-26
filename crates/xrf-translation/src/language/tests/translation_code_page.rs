use xrf_utils::{new_windows1250_encoder, new_windows1251_encoder, new_windows1252_encoder};

use crate::language::translation_code_page::TranslationCodePage;

#[test]
fn finds_the_character_an_encoding_cannot_hold() {
  assert_eq!(
    TranslationCodePage::find_unencodable("plain ascii", new_windows1252_encoder()),
    None
  );
  assert_eq!(
    TranslationCodePage::find_unencodable("Привет", new_windows1252_encoder()),
    Some('П')
  );
  // Cyrillic is not in 1250 either, which is what makes a Polish target refuse it.
  assert_eq!(
    TranslationCodePage::find_unencodable("Й", new_windows1250_encoder()),
    Some('Й')
  );
  // The first one, not merely some, so a refusal names the character a reader meets first.
  assert_eq!(
    TranslationCodePage::find_unencodable("ok, José and Jürgen", new_windows1251_encoder()),
    Some('é')
  );
}

#[test]
fn describes_a_character_by_its_code_point() {
  assert_eq!(TranslationCodePage::describe('é'), "'é' (U+00E9)");
}
