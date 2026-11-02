use crate::text::translation_edit::TranslationEdit;
use crate::text::translation_variant::TranslationVariant;

#[test]
fn an_edit_reports_the_id_it_targets() {
  assert_eq!(
    TranslationEdit::Set {
      id: String::from("st_a"),
      value: TranslationVariant::String(String::from("x")),
    }
    .id(),
    "st_a"
  );
  assert_eq!(
    TranslationEdit::Remove {
      id: String::from("st_b")
    }
    .id(),
    "st_b"
  );
}
